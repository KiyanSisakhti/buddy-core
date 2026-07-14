#![no_std]

//! # Buddy Allocator Core (`#![no_std]`)
//!
//! A highly optimized, compile-time configurable **Binary Buddy Allocator** engine designed specifically
//! for bare-metal systems, custom OS kernels, and embedded environments where a standard library (`std`)
//! is unavailable.
//!
//! ## Architecture Overview
//!
//! The allocator is structurally split into three distinct layers of abstraction:
//!
//! 1. **[`BuddyBase`] (The Brain):** Coordinates the overall system. It handles high-level allocation requests,
//!    implements recursive buddy splitting, and triggers cascading buddy mergers (coalescing) when blocks are freed.
//! 2. **[`BuddyOrder`] (The Layer Manager):** Manages a doubly-linked list of free blocks representing a specific
//!    power-of-two size class.
//! 3. **[`IBuddyMdAdapter`] & [`IBuddyMetaData`] (The Hardware/Data Bridges):** Decouples the physical location
//!    of metadata from the allocator's logical operations. This allows metadata to be stored alongside the blocks
//!    (intrusive), in a flat global array, or even in a completely custom page-table-like structure.
//!
//! ---
//!
//! ## Designing a Custom Metadata Adapter
//!
//! To plug this engine into your system, you must implement two traits:
//! * [`IBuddyMetaData`]: Defines how to read/write intrusive node linkages (`next`, `last`, `order`, etc.).
//! * [`IBuddyMdAdapter`]: Defines how to retrieve a metadata handle from a raw physical block address (`u64`).
//!
//! # Examples
//!
//! Below is a comprehensive blueprint demonstrating how to implement the required traits using a mock static
//! array for block metadata, and how to initialize and drive [`BuddyBase`].
//!
//!
//! ```rust
//! use buddy_core::{BuddyBase, IBuddyMdAdapter, IBuddyMetaData, BuddyError};
//!
//! // 1. Define our Node Metadata structure that actually lives in memory
//! #[derive(Clone, Copy)]
//! pub struct MyPageMetadata {
//!     next: Option<u64>,
//!     last: Option<u64>,
//!     order: u8,
//!     ceil_reduction: u8,
//!     is_linked: bool,
//! }
//!
//! // 2. Define a Handle that wraps a raw pointer to the metadata.
//! // This ensures modifications write back to the original table.
//! #[derive(Clone, Copy)]
//! pub struct MyPageMetadataHandle {
//!     ptr: *mut MyPageMetadata,
//! }
//!
//! // 3. Implement the IBuddyMetaData interface by defining the Associated Type inside the body
//! pub struct MyMetaInterface;
//! impl IBuddyMetaData for MyMetaInterface {
//!     type MetaData = MyPageMetadataHandle;
//!
//!     fn get_next(md: &Self::MetaData) -> Option<u64> {
//!         unsafe { (*md.ptr).next }
//!     }
//!     fn set_next(md: &mut Self::MetaData, next: Option<u64>) {
//!         unsafe { (*md.ptr).next = next; }
//!     }
//!     fn get_last(md: &Self::MetaData) -> Option<u64> {
//!         unsafe { (*md.ptr).last }
//!     }
//!     fn set_last(md: &mut Self::MetaData, last: Option<u64>) {
//!         unsafe { (*md.ptr).last = last; }
//!     }
//!     fn get_order(md: &Self::MetaData) -> u8 {
//!         unsafe { (*md.ptr).order }
//!     }
//!     fn set_order(md: &mut Self::MetaData, order: u8) {
//!         unsafe { (*md.ptr).order = order; }
//!     }
//!     fn get_ceil_reduction(md: &Self::MetaData) -> u8 {
//!         unsafe { (*md.ptr).ceil_reduction }
//!     }
//!     fn set_ceil_reduction(md: &mut Self::MetaData, reduct: u8) {
//!         unsafe { (*md.ptr).ceil_reduction = reduct; }
//!     }
//!     fn is_linked(md: &Self::MetaData) -> bool {
//!         unsafe { (*md.ptr).is_linked }
//!     }
//!     fn set_link(md: &mut Self::MetaData, linked: bool) {
//!         unsafe { (*md.ptr).is_linked = linked; }
//!     }
//! }
//!
//! // 4. Set up a global metadata storage
//! static mut METADATA_TABLE: [MyPageMetadata; 1024] = [MyPageMetadata {
//!     next: None,
//!     last: None,
//!     order: 0,
//!     ceil_reduction: 0,
//!     is_linked: false,
//! }; 1024];
//!
//! // 5. Implement the Adapter to return the pointer-based Handle
//! pub struct MySystemAdapter;
//! impl IBuddyMdAdapter for MySystemAdapter {
//!     type MetaDataHandle = MyPageMetadataHandle;
//!     type Interface = MyMetaInterface;
//!
//!     fn get_md(n: u64) -> Option<Self::MetaDataHandle> {
//!         let index = n as usize;
//!         if index < 1024 {
//!             unsafe {
//!                 let ptr = core::ptr::addr_of_mut!(METADATA_TABLE[index]);
//!                 Some(MyPageMetadataHandle { ptr })
//!             }
//!         } else {
//!             None
//!         }
//!     }
//! }
//!
//! // 6. Drive the Buddy Allocator in a standard execution flow
//! fn main() -> Result<(), BuddyError> {
//!     let mut allocator = BuddyBase::<MySystemAdapter, 8>::new();
//!
//!     allocator.push_with_order(0, 0, 0)?;
//!     allocator.push_with_order(1, 0, 0)?;
//!
//!     let allocated_block = allocator.pop(1)?;
//!     assert_eq!(allocated_block, 0);
//!
//!     Ok(())
//! }
//! ```

mod buddy_base;
mod buddy_err;
mod buddy_md_adapter_interface;
mod buddy_md_interface;
mod buddy_order;
mod utils;

pub use buddy_base::BuddyBase;
pub use buddy_err::BuddyError;
pub use buddy_md_adapter_interface::IBuddyMdAdapter;
pub use buddy_md_interface::IBuddyMetaData;
pub use utils::allocate_ceil_reductor;
