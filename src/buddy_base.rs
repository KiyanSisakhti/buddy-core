/// The orchestrator and core brain of the Buddy Allocator system.
///
/// `BuddyBase` manages an array of [`BuddyOrder`] layers, routing allocation (`pop`)
/// and deallocation (`push`) requests across different order levels. It implements the two
/// fundamental algorithms of a binary buddy system: **Recursive Coalescing** (merging adjacent free
/// buddies upwards) and **Recursive Splitting** (breaking down larger blocks downwards).
///
/// # Compile-time Safety Constraints
/// * `ORDER_COUNT` - The maximum order depth of the allocator (bounded at compile time up to 31).
/// * `Adapter` - The interface wrapper that bridges the allocator to the target physical metadata.
///
use crate::{
    IBuddyMdAdapter, IBuddyMetaData,
    buddy_err::BuddyError,
    buddy_order::BuddyOrder,
    utils::{buddy_lookup, is_aligned_at_order},
};

pub struct BuddyBase<Adapter, const ORDER_COUNT: usize = 2>
where
    Adapter: IBuddyMdAdapter,
{
    /// Array containing the tracker state for every individual order layer.
    orders: [BuddyOrder<ORDER_COUNT, Adapter>; ORDER_COUNT],
}

impl<Adapter, const ORDER_COUNT: usize> BuddyBase<Adapter, ORDER_COUNT>
where
    Adapter: IBuddyMdAdapter,
{
    /// Compile-time guard asserting that `ORDER_COUNT` does not exceed 31.
    /// This prevents arithmetic overflows on 32-bit/64-bit systems during pointer calculation.
    const MAX_CHECK: () = {
        assert!(ORDER_COUNT <= 31);
    };

    /// Creates a initialized instance of the buddy allocator controller.
    pub fn new() -> Self {
        let _: () = Self::MAX_CHECK;
        Self {
            orders: core::array::from_fn(|d| BuddyOrder::new(d as u8)),
        }
    }

    /// Helper utility to convert a target maximum order ceiling into a relative "reduction index".
    ///
    /// This defines how many layers below `ORDER_COUNT` a memory block's merging potential must stop.
    #[deprecated(
        since = "0.1.0",
        note = "Please use 'buddy_core::allocate_ceil_reductor' instead."
    )]
    #[inline]
    pub fn allocate_ceil_reductor(max_order: u8) -> u8 {
        (ORDER_COUNT as u8 - 1) - max_order
    }

    /// Safely introduces a raw page/block into the system with an assigned order and merge ceiling.
    ///
    /// This is typically invoked during allocator bootstrap/initialization when raw memory regions
    /// are mapped to the core.
    ///
    /// # Errors
    /// * [`BuddyError::AlignmentMismatch`] - If the block address `n` isn't aligned to its starting order boundary.
    /// * [`BuddyError::DataCorrupted`] - If the metadata handle lookup fails.
    pub fn push_with_order(
        &mut self,
        n: u64,
        order: u8,
        ceil_reduct: u8,
    ) -> Result<(), BuddyError> {
        if !is_aligned_at_order(n, order) {
            return Err(BuddyError::AlignmentMismatch);
        }

        let ceil_max = (ORDER_COUNT as u8 - 1) - ceil_reduct;
        if order > ceil_max {
            return Err(BuddyError::AlignmentMismatch);
        }

        let Some(mut md) = Adapter::get_md(n) else {
            return Err(BuddyError::DataCorrupted);
        };
        Adapter::Interface::set_ceil_reduction(&mut md, ceil_reduct);
        Adapter::Interface::set_order(&mut md, order);

        self.push(n)
    }

    /// Frees an active memory block back to the allocator, triggering the coalesce engine.
    ///
    /// # Process
    /// 1. Determines the block's current active `order` and maximum allowed `ceil_reduction`.
    /// 2. Performs safety checks to verify alignment and guard against catastrophic double-frees.
    /// 3. Hands off control to the recursive `insert_fix` pipeline to execute buddy merges.
    pub fn push(&mut self, n: u64) -> Result<(), BuddyError> {
        let Some(md) = Adapter::get_md(n) else {
            return Err(BuddyError::DataCorrupted);
        };

        let ord = Adapter::Interface::get_order(&md);
        let ceil_reduct = Adapter::Interface::get_ceil_reduction(&md);

        if !is_aligned_at_order(n, ord) {
            return Err(BuddyError::DataCorrupted);
        }

        // Proactively probe the companion buddy's metadata to detect illegal double-frees
        let buddy_n = buddy_lookup(n, ord);
        if let Some(buddy_md) = Adapter::get_md(buddy_n)
            && Adapter::Interface::get_order(&buddy_md) > ord
        {
            return Err(BuddyError::DoubleFree);
        };

        self.insert_fix(n, ord, ceil_reduct)
    }

    /// Core recursive engine that merges freed blocks with their matching "buddies".
    ///
    /// This function acts as a ripple-effect solver:
    /// - It computes the expected buddy identifier at the current `order`.
    /// - It attempts to locate and unlink that buddy from the current free list.
    /// - **If found:** The buddy is unlinked, and both are unified under the lower numerical
    ///   index (`n.min(buddy_n)`), which is then recursively sent up to `order + 1`.
    /// - **If not found:** The coalescence terminates, and the block is safely pushed into
    ///   the current order's free list.
    ///
    /// Recursion automatically stops once the block hits its custom ceiling limit (`ceiled_max_ord`).
    fn insert_fix(&mut self, n: u64, order: u8, ceil_reductor: u8) -> Result<(), BuddyError> {
        let bd_ord = &mut self.orders[order as usize];
        let ceiled_max_ord = (ORDER_COUNT as u8) - ceil_reductor;

        // Base Case: If the next level exceeds our structural boundary limit, push and terminate
        if (order + 1) >= ceiled_max_ord {
            return bd_ord.push(n);
        };

        let buddy_n = buddy_lookup(n, order);

        // Attempt to capture and extract the buddy block from its active free list
        if let Err(err) = bd_ord.try_remove_at_order(buddy_n, order + 1) {
            match err {
                BuddyError::NotFound => {
                    // Buddy is busy (allocated), so we can't merge. Rest here.
                    return bd_ord.push(n);
                }
                _ => return Err(err), // Structural corruption or alignment error cascade
            }
        }

        // Buddy successfully unlinked! Merge them into a single block at the next higher order.
        let min = n.min(buddy_n);
        self.insert_fix(min, order + 1, ceil_reductor)
    }

    /// Allocates a block at the requested `order`.
    ///
    /// # Allocation Pipeline
    /// 1. Tries to pop a block from the target order's free list immediately.
    /// 2. If empty ([`BuddyError::NotFound`]), it invokes [`buddy_emission`] to look up higher layers.
    /// 3. Once a block is retrieved (or split and bubbled down), it configures the metadata
    ///    and returns the block address.
    pub fn pop(&mut self, order: u8) -> Result<u64, BuddyError> {
        let bd_ord = &mut self.orders[order as usize];

        let num = match bd_ord.pop(order) {
            Ok(n) => n,
            Err(BuddyError::NotFound) => self.buddy_emission(order)?,
            Err(err) => return Err(err),
        };

        let mut md = Adapter::get_md(num).ok_or(BuddyError::DataCorrupted)?;
        Adapter::Interface::set_order(&mut md, order);

        Ok(num)
    }

    /// Performs a top-down recursive split of a larger block to satisfy a lower-order request.
    ///
    /// This is the classic **Buddy Splitting** mechanism:
    /// - If no block is available in the current order, it asks the higher level (`nxt_trg`) for a block.
    /// - This ripples upwards until a free block is found (or it fails, returning `NotFound`).
    /// - Once a higher-order block is retrieved, it is split into two buddies:
    ///   - The "Left" buddy (index `n`) is returned to satisfy the lower-order pipeline.
    ///   - The "Right" buddy (index `bd`) is pushed down into the current layer's free list.
    fn buddy_emission(&mut self, targ: u8) -> Result<u64, BuddyError> {
        let nxt_trg = targ + 1;
        if nxt_trg >= ORDER_COUNT as u8 {
            return Err(BuddyError::NotFound);
        }

        // Recurse upwards to fetch a larger block to split
        let n = match self.orders[nxt_trg as usize].pop(targ) {
            Ok(n) => n,
            Err(BuddyError::NotFound) => self.buddy_emission(nxt_trg)?,
            Err(err) => return Err(err),
        };

        // Calculate the buddy counterpart for the split block
        let bd = buddy_lookup(n, targ);

        // Store the second half (the companion) in our current lower order layer
        self.orders[targ as usize].push(bd)?;

        // Return the first half downwards
        Ok(n)
    }

    // Using on debug
    // pub fn dump(&self) {
    //     for t in &self.orders {
    //         println!("\nl {}:", t.order);
    //         t.dump();
    //     }
    // }
}

impl<Adapter, const ORDER_COUNT: usize> Default for BuddyBase<Adapter, ORDER_COUNT>
where
    Adapter: IBuddyMdAdapter,
{
    fn default() -> Self {
        Self {
            orders: core::array::from_fn(|d| BuddyOrder::new(d as u8)),
        }
    }
}
