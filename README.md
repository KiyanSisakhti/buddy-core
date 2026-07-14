<div align="center">
<h1>Buddy Core</h1>

<p><b>A generic, <code>#![no_std]</code>, allocation-free Binary Buddy Allocator engine for kernels and embedded systems.</b></p>

</div>



<div align="center">

![Test Status][git-ci]
![Lines of Code][total-lines]
![Repo Size][repo-size]


[![crate][crate-badge]][crate-link]
[![MIT licensed][license-image]][license-link]
[![Docs][docs-image]][docs-link]

</div>


---

## 📖 What is this?

`buddy-core` is **not** a ready-to-use page allocator. It is the *engine* — the pure algorithmic
core of a binary buddy allocator, completely decoupled from where and how block metadata
is actually stored.

Think of it like this:

- **You** decide where the metadata lives (a static array, a slab, a page-table-like structure, intrusive
  headers inside the block itself — anything).
- **`buddy-core`** decides *when* to split blocks, *when* to merge (coalesce) them, and *how* to
  keep track of free lists per order — the actual buddy-system logic.

This separation is done through two small traits (`IBuddyMdAdapter` and `IBuddyMetaData`), which act
as a bridge between the allocator's logic and your storage of choice. Because of this, `buddy-core`
has **zero dependency on `alloc`** and works in the most constrained bare-metal environments —
OS kernels, bootloaders, and embedded firmware.

---

## 🧠 The Core Idea: Buddy Allocation

A binary buddy allocator manages memory in power-of-two sized blocks, organized into "orders":

```
Order 0 → block size 1 unit
Order 1 → block size 2 units
Order 2 → block size 4 units
Order 3 → block size 8 units
...
Order N → block size 2^N units
```

Every block at order `o` has exactly one **buddy**: the adjacent block of the same size that,
together with it, forms a single block at order `o + 1`. The buddy's address is computed with a
single XOR:

```
buddy(n, order) = n ^ (1 << order)
```

This is the mathematical trick that makes buddy allocators fast — no searching required to find
a block's partner, just one bitwise operation.

### Splitting (on allocation)

If you ask for a block at order `k` and none is free, the allocator recursively climbs to the
next order up looking for a free block. Once found, it **splits** that block in half:

```
Order 3 block [0..8)         "found free"
        │
        ▼ split
Order 2 block [0..4)   Order 2 block [4..8)
   (returned to you)     (pushed back as free)
```

If the order-2 half is still bigger than requested, it keeps splitting downward until it
reaches the requested order.

### Coalescing (on free)

When you free a block, the allocator checks whether its buddy is *also* free. If it is, they
merge back into a single block at the next order up — and the process repeats recursively,
bubbling upward, until a busy buddy is found or the top order is reached.

```
Free block 4 (order 2)
        │
   is buddy (block 0, order 2) free? → yes
        │
        ▼ merge
   Order 3 block [0..8) is now free
        │
   is buddy (block 8, order 3) free? → check again...
```

This recursive merge is what keeps external fragmentation low over time.

---

## 🏗 Architecture

```
┌───────────────────────────────────────────────────────────┐
│                        BuddyBase                          │
│   The orchestrator. Owns one BuddyOrder per order level.  │
│   Implements recursive splitting (buddy_emission) and     │
│   recursive coalescing (insert_fix).                      │
└───────────────────────┬───────────────────────────────────┘
                         │ owns [ BuddyOrder; ORDER_COUNT ]
                         ▼
┌───────────────────────────────────────────────────────────┐
│                       BuddyOrder                          │
│   Manages ONE order's free list (a doubly linked list).   │
│   push() / pop() / try_remove_at_order() — head and       │
│   middle-of-list insert/extract operations.               │
└───────────────────────┬───────────────────────────────────┘
                         │ reads/writes node state via
                         ▼
┌───────────────────────────────────────────────────────────┐
│         IBuddyMdAdapter  +  IBuddyMetaData                │
│   Your bridge. You decide WHERE metadata for a block      │
│   index lives (static array, intrusive header, etc.) and  │
│   provide get/set for: next, last, order, ceil_reduction, │
│   is_linked.                                              │
└───────────────────────────────────────────────────────────┘
```

### `BuddyBase<Adapter, ORDER_COUNT>` — the brain

- `new()` — builds `ORDER_COUNT` independent `BuddyOrder` layers.
- `push_with_order(addr, order, ceil_reduct)` — bootstraps a raw block into the system at a given
  order, with an optional "ceiling" limiting how far it's allowed to merge upward (useful for
  carving reserved zones that shouldn't blend with other memory regions).
- `push(addr)` — frees a block, verifies alignment, detects double-frees by checking the buddy's
  state, and hands off to `insert_fix` for recursive coalescing.
- `pop(order)` — allocates a block at the given order. Tries the direct free list first, and if
  empty, calls `buddy_emission` to recursively split a larger block down.
- `insert_fix` (private) — the recursive merge engine. Looks up the buddy, tries to unlink it from
  its free list; if found, merges and recurses one order higher; if not found (buddy is busy),
  the block is simply pushed into its own free list.
- `buddy_emission` (private) — the recursive split engine. Climbs orders upward until a free block
  is found, then splits it back down, pushing the leftover half into the free list at each step.

### `BuddyOrder<ORDER_COUNT, Adapter>` — a single order's free list

A hand-rolled doubly linked list over your metadata, addressed by raw `u64` block index instead
of pointers:

- `push(n)` — inserts at the head. Errors on double-free (`is_linked` already `true`).
- `pop(target_order)` — removes the head, bumps its recorded order to `target_order`.
- `try_remove_at_order(n, target_order)` — dispatches to either `pop` (if `n` is the head) or
  `try_unlink_at_order` (if `n` is somewhere in the middle of the list), so merges can grab a
  buddy no matter where it sits in the list.
- `try_unlink_at_order` (private) — classic doubly-linked-list middle-removal: stitches the
  neighbors' `next`/`last` pointers together.
- `can_be_at_order` (private) — a ceiling check ensuring a block isn't merged past its assigned
  reduction limit.

### `IBuddyMdAdapter` / `IBuddyMetaData` — the storage bridge

Two traits you implement once for your environment:

```rust
pub trait IBuddyMetaData {
    type MetaData;
    fn get_next(md: &Self::MetaData) -> Option<u64>;
    fn set_next(md: &mut Self::MetaData, n: Option<u64>);
    fn get_last(md: &Self::MetaData) -> Option<u64>;
    fn set_last(md: &mut Self::MetaData, n: Option<u64>);
    fn get_order(md: &Self::MetaData) -> u8;
    fn set_order(md: &mut Self::MetaData, order: u8);
    fn get_ceil_reduction(md: &Self::MetaData) -> u8;
    fn set_ceil_reduction(md: &mut Self::MetaData, ceil_reduct: u8);
    fn is_linked(md: &Self::MetaData) -> bool;
    fn set_link(md: &mut Self::MetaData, state: bool);
}

pub trait IBuddyMdAdapter {
    type Interface: IBuddyMetaData<MetaData = Self::MetaDataHandle>;
    type MetaDataHandle;
    fn get_md(n: u64) -> Option<Self::MetaDataHandle>;
}
```

**Why a "handle" and not a direct reference?** Methods take `&Self::MetaData` (a reference *to
the handle*, not to the raw struct). If your handle is itself `&'static mut SomeStruct`, this
becomes a reference-to-a-mutable-reference. That extra layer of indirection is what lets the
allocator read and write several different blocks' metadata in the same scope (e.g. a block and
its buddy) without the borrow checker treating it as two live mutable borrows of the same data —
all without runtime-checked wrappers like `RefCell`.

---

## 🚀 Quick Start

```rust
use buddy_core::{BuddyBase, IBuddyMdAdapter, IBuddyMetaData, BuddyError};

// 1. Your metadata struct — however you want to store per-block state.
#[derive(Clone, Copy)]
pub struct PageMeta {
    next: Option<u64>,
    last: Option<u64>,
    order: u8,
    ceil_reduction: u8,
    is_linked: bool,
}

// 2. A handle wrapping a raw pointer, so writes go back to the original table.
#[derive(Clone, Copy)]
pub struct PageMetaHandle { ptr: *mut PageMeta }

// 3. Implement the get/set interface.
pub struct PageMetaInterface;
impl IBuddyMetaData for PageMetaInterface {
    type MetaData = PageMetaHandle;
    fn get_next(md: &Self::MetaData) -> Option<u64> { unsafe { (*md.ptr).next } }
    fn set_next(md: &mut Self::MetaData, n: Option<u64>) { unsafe { (*md.ptr).next = n; } }
    fn get_last(md: &Self::MetaData) -> Option<u64> { unsafe { (*md.ptr).last } }
    fn set_last(md: &mut Self::MetaData, n: Option<u64>) { unsafe { (*md.ptr).last = n; } }
    fn get_order(md: &Self::MetaData) -> u8 { unsafe { (*md.ptr).order } }
    fn set_order(md: &mut Self::MetaData, order: u8) { unsafe { (*md.ptr).order = order; } }
    fn get_ceil_reduction(md: &Self::MetaData) -> u8 { unsafe { (*md.ptr).ceil_reduction } }
    fn set_ceil_reduction(md: &mut Self::MetaData, r: u8) { unsafe { (*md.ptr).ceil_reduction = r; } }
    fn is_linked(md: &Self::MetaData) -> bool { unsafe { (*md.ptr).is_linked } }
    fn set_link(md: &mut Self::MetaData, s: bool) { unsafe { (*md.ptr).is_linked = s; } }
}

// 4. Global storage for 1024 blocks' metadata.
static mut TABLE: [PageMeta; 1024] = [PageMeta {
    next: None, last: None, order: 0, ceil_reduction: 0, is_linked: false,
}; 1024];

// 5. The adapter maps a raw block index -> a handle into that table.
pub struct SystemAdapter;
impl IBuddyMdAdapter for SystemAdapter {
    type Interface = PageMetaInterface;
    type MetaDataHandle = PageMetaHandle;
    fn get_md(n: u64) -> Option<Self::MetaDataHandle> {
        let i = n as usize;
        if i < 1024 {
            unsafe { Some(PageMetaHandle { ptr: core::ptr::addr_of_mut!(TABLE[i]) }) }
        } else {
            None
        }
    }
}

fn main() -> Result<(), BuddyError> {
    // ORDER_COUNT = 8 → orders 0..=7 supported.
    let mut allocator = BuddyBase::<SystemAdapter, 8>::new();

    // Seed the allocator with two raw blocks at order 0.
    allocator.push_with_order(0, 0, 0)?;
    allocator.push_with_order(1, 0, 0)?;

    // Because both blocks 0 and 1 are buddies at order 0, they auto-merge into
    // a single order-1 block during push_with_order's internal push().
    let block = allocator.pop(1)?; // pop a full order-1 (2-unit) block
    assert_eq!(block, 0);

    Ok(())
}
```

---

## 🔑 Key Concepts Explained Simply

| Concept | Plain-English Explanation |
|---|---|
| **Order** | A "size class". Order `o` means a block of size `2^o`. Order 0 is the smallest unit. |
| **Buddy** | The specific neighboring block that combines with yours to form the next-size-up block. Found instantly via XOR, no search needed. |
| **Splitting** | Breaking a big free block into two smaller (buddy) blocks when a smaller size is requested and none is directly available. |
| **Coalescing** | Merging a freed block with its buddy (if also free) into a bigger block, recursively, to fight fragmentation. |
| **Ceiling / `ceil_reduction`** | An optional cap that stops a block from merging past a certain order — useful for keeping separate memory regions (e.g. DMA-safe memory vs normal RAM) from blending together during coalescing. |
| **Metadata Adapter** | The plug-in point where *you* tell the allocator how to fetch a block's bookkeeping data (order, links, etc.) from wherever *you* choose to store it. |
| **Double-free protection** | Before merging, the allocator peeks at the buddy's recorded order. If the buddy looks "already free at a higher order" than expected, that's a red flag for a double free, and it errors out instead of corrupting state. |

---

## 🧱 Deep Dive: `ceil_reduction` — pinning a block below a certain order

Normally, when a block is freed, `insert_fix` keeps merging it with its buddy, upward,
order after order, for as long as a free buddy is available. Left unchecked, this means
**any** two adjacent free blocks can eventually combine into the largest possible block size —
even if they came from two logically different memory regions that just happen to sit next
to each other in address space.

`ceil_reduction` is how you tell the allocator: *"this specific block is allowed to merge
upward, but only up to this order — never past it."* It acts as a personal ceiling, carried
in the block's own metadata, checked on every merge attempt.

### How it's computed

```rust
let ceiled_max_ord = (ORDER_COUNT as u8) - ceil_reductor;
```

So `ceil_reduct` isn't the ceiling order itself — it's *how many orders below `ORDER_COUNT`*
the ceiling sits. A `ceil_reduct` of `0` means "no restriction, allowed to merge all the way
to the top order." A larger `ceil_reduct` pulls that ceiling further down.

This check happens in two places:

- **`BuddyBase::insert_fix`** — before recursing to `order + 1`, it checks
  `(order + 1) >= ceiled_max_ord`. If the next merge would cross the ceiling, it stops and
  pushes the block into its current order's free list instead of climbing further.
- **`BuddyOrder::can_be_at_order`** — a second, symmetric guard used during `pop`/unlink, making
  sure a block being pulled out for a merge is actually still allowed to exist at the
  target order: `(target_order + ceil_reduct) < ORDER_COUNT`.

### Worked example

Say `ORDER_COUNT = 13` (orders 0..=12 exist, order 12 is the biggest). You have two separate
memory zones sitting next to each other in address space — a small reserved DMA zone and the
regular general-purpose zone — and you don't want blocks from one ever coalescing into the
other:

```rust
// General-purpose zone: free to merge all the way up to order 12.
allocator.push_with_order(0, 12, 0)?;   // ceil_reduct = 0 -> ceiling = order 12 (no cap)

// DMA-reserved zone: only 3 pages (order 0..=9 sized blocks), never allowed to
// grow into a block bigger than order 9.
allocator.push_with_order(15360, 9, 3)?; // ceil_reduct = 3 -> ceiling = 13 - 3 = order 10
```

For the second block: even if its buddy is free and merging would normally continue, the
moment the recursive merge in `insert_fix` would produce a block at order `10` or higher
(`(order + 1) >= ceiled_max_ord`, i.e. `>= 10`), it stops climbing and simply rests as a free
block at whatever order it reached — order 9 at most. It can never accidentally fuse with a
neighboring block from the general-purpose zone into something order 10 or larger.

### Why this matters in practice

- **Zone isolation** — keep DMA-safe, NUMA-local, or otherwise "special" memory regions from
  silently merging with regular memory during coalescing.
- **Bounded worst case** — you can guarantee a region never grows past a known maximum block
  size, which matters for allocators that reserve fixed-size regions for specific subsystems.
- **No extra bookkeeping cost** — the ceiling travels with the block's own metadata
  (`get_ceil_reduction` / `set_ceil_reduction`), so there's no global table or extra pass
  needed to enforce it; it's checked as a cheap arithmetic comparison right inside the
  existing merge/split recursion.

## 🧪 Testing

The crate ships with a logic + fuzz test (`tests/logic_test.rs` in the example test suite) that:

1. **Sets up a realistic mixed allocator** — nine blocks pushed at different orders (12, 11, 11,
   12, 10, 10, 10, 9, 9) with matching `ceil_reduct` values, simulating a real memory layout with
   segmented zones.
2. **Drains everything down to order 0**, shuffles the resulting addresses (to remove any
   ordering bias), then frees them all back — exercising the full split/merge pipeline under
   randomized conditions.
3. **Asserts `alloc_count == dealloc_count`** — every allocation must be matched by exactly one
   deallocation, proving no blocks are silently lost or duplicated.
4. **Exhaustively drains each top order** (12 → 9) and asserts the *exact* expected block count
   comes out before hitting `BuddyError::NotFound` — proving the coalescing pipeline correctly
   reassembled all fragments back into their original large blocks.

```rust
fn setup_initial_allocator() -> BuddyBase<TestMetaDataHandler, 13> {
    let mut alloc = BuddyBase::new();
    alloc.push_with_order(0,     12, 0).unwrap();
    alloc.push_with_order(4096,  11, 1).unwrap();
    alloc.push_with_order(6144,  11, 1).unwrap();
    alloc.push_with_order(8192,  12, 0).unwrap();
    alloc.push_with_order(12288, 10, 2).unwrap();
    alloc.push_with_order(13312, 10, 2).unwrap();
    alloc.push_with_order(14336, 10, 2).unwrap();
    alloc.push_with_order(15360, 9,  3).unwrap();
    alloc.push_with_order(15872, 9,  3).unwrap();
    alloc
}

#[test]
pub fn logic_test() {
    let mut alloc = setup_initial_allocator();

    // Drain everything to order 0, shuffle, then give it all back —
    // stress-testing split then merge from a randomized state.
    let mut allocated = Vec::new();
    while let Ok(addr) = alloc.pop(0) {
        allocated.push(addr);
    }
    allocated.shuffle();
    for addr in allocated.drain(..) {
        alloc.push(addr).unwrap();
    }

    // Fuzz: thousands of randomized alloc/free pairs.
    let generator = bolero::produce::<Vec<(u8, u8)>>().with().len(1000..2000);
    let (mut alloc_count, mut dealloc_count) = (0u64, 0u64);

    bolero::check!().with_generator(generator).for_each(|ops| {
        let mut live = Vec::new();
        for &(op, raw_order) in ops {
            if op % 2 == 0 {
                if let Ok(addr) = alloc.pop(raw_order % 13) {
                    live.push(addr);
                    alloc_count += 1;
                }
            } else if let Some(addr) = live.pop() {
                alloc.push(addr).unwrap();
                dealloc_count += 1;
            }
        }
        for addr in live.drain(..) {
            alloc.push(addr).unwrap();
            dealloc_count += 1;
        }
    });

    assert_eq!(alloc_count, dealloc_count);

    // Sanity check: after the fuzz run settles, the original top-level
    // blocks should be fully reassembled and poppable exactly as seeded.
    alloc.pop(12).unwrap();
    alloc.pop(12).unwrap();
    assert_eq!(alloc.pop(12).unwrap_err(), BuddyError::NotFound);

    alloc.pop(11).unwrap();
    alloc.pop(11).unwrap();
    assert_eq!(alloc.pop(11).unwrap_err(), BuddyError::NotFound);

    alloc.pop(10).unwrap();
    alloc.pop(10).unwrap();
    alloc.pop(10).unwrap();
    assert_eq!(alloc.pop(10).unwrap_err(), BuddyError::NotFound);

    alloc.pop(9).unwrap();
    alloc.pop(9).unwrap();
    assert_eq!(alloc.pop(9).unwrap_err(), BuddyError::NotFound);
}
```

Run it with:

```bash
cargo test
```


---

## ⚠️ Error Reference

| Variant | Meaning |
|---|---|
| `BuddyError::DoubleFree` | A block was freed while already marked as linked/free — classic double-free bug caught before it can corrupt the free list. |
| `BuddyError::DataCorrupted` | A metadata lookup failed (`get_md` returned `None`) where a valid handle was expected — usually points to an out-of-bounds or invalid index. |
| `BuddyError::NotFound` | No free block available at the requested order, and nothing higher could be split (allocator is out of memory at that size), or a buddy couldn't be located for a merge/unlink. |
| `BuddyError::AlignmentMismatch` | The address isn't aligned to its order's boundary, or the order requested exceeds the block's allowed ceiling. |

---

## 📦 Installation

```toml
[dependencies]
buddy-core = "0.1.0"
```

`edition = "2024"`, `#![no_std]`, no `alloc` dependency.

---

## 📄 License

This project is licensed under the **MIT License**. See the `LICENSE` file for details.

[crate-badge]: https://img.shields.io/crates/v/buddy-core.svg
[crate-link]: https://crates.io/crates/buddy-core
[docs-image]: https://docs.rs/buddy-core/badge.svg
[docs-link]: https://docs.rs/buddy-core
[license-image]: https://img.shields.io/badge/MIT-blue.svg
[repo-size]: https://img.shields.io/github/repo-size/KiyanSisakhti/buddy-core
[total-lines]: https://aschey.tech/tokei/github/KiyanSisakhti/buddy-core
[git-ci]:https://github.com/KiyanSisakhti/buddy-core/actions/workflows/rust.yml/badge.svg?branch=main

[license-link]: #license
