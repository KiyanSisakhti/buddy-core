//! A behavior trait defining how to read and write structural states of a block's metadata.
//!
//! This trait provides a unified API for the buddy allocator to query and modify state fields
//! (such as linked list pointers, current buddy orders, and ceil reduction limits)
//! of individual memory blocks.
//!
//! # Crucial Architectural Detail: Handle Passing vs. Direct References
//! A subtle but incredibly powerful aspect of this trait is that its methods **do not** take
//! a direct reference to the raw metadata struct (e.g., `&RawMetadata`). Instead, they accept
//! a reference to the **Metadata Handle** (`&Self::MetaData`).
//!
//! ### Why this is a game-changer for Rust's Borrow Checker:
//! 1. **Zero-Cost Abstraction over Indirection:** If your `MetaData` handle is defined as a mutable
//!    static reference (like `&'static mut TestMetaData`), the method signature `&Self::MetaData`
//!    evaluates to a double reference: `&&'static mut TestMetaData`.
//! 2. **Avoiding Lifetime Aliasing Violations:** The buddy allocator frequently needs to read
//!    from and write to multiple block metadatas (e.g., during buddy lookup and unlinking) inside
//!    the same scope. If we passed raw references, Rust's borrow checker would flag this as
//!    multiple simultaneous mutable borrows.
//! 3. **Indirection Safety:** Passing a reference to the handle (`&Self::MetaData`) bypasses these
//!    borrow-checking restrictions, allowing the allocator to perform fast, inline pointer
//!    manipulations without wrapping metadata in expensive runtime-checked wrappers like `RefCell`.

pub trait IBuddyMetaData {
    /// The container or reference pointer type representing the metadata handle.
    ///
    /// This associated type is typically bound to the `MetaDataHandle` of your adapter.
    /// In highly optimized bare-metal setups, this is often `&'static mut YourMetadataStruct`.
    type MetaData;

    /// Retrieves the index (`u64`) of the next block in the current order's doubly linked list.
    ///
    /// Returns `None` if this block is the last node or not in a list.
    fn get_next(md: &Self::MetaData) -> Option<u64>;

    /// Sets the index of the next block in the doubly linked list.
    fn set_next(md: &mut Self::MetaData, n: Option<u64>);

    /// Retrieves the index (`u64`) of the previous (last) block in the doubly linked list.
    ///
    /// Returns `None` if this block is the head node or not in a list.
    fn get_last(md: &Self::MetaData) -> Option<u64>;

    /// Sets the index of the previous (last) block in the doubly linked list.
    fn set_last(md: &mut Self::MetaData, n: Option<u64>);

    /// Updates the current buddy allocation order of the block.
    fn set_order(md: &mut Self::MetaData, order: u8);

    /// Retrieves the current buddy allocation order of the block.
    fn get_order(md: &Self::MetaData) -> u8;

    /// Gets the custom upper limit reduction (ceiling) of this block.
    ///
    /// This value is used to calculate the maximum order this block is allowed to merge into,
    /// preventing heterogeneous memory zones from overlapping.
    fn get_ceil_reduction(md: &Self::MetaData) -> u8;

    /// Sets the custom upper limit reduction (ceiling) of this block.
    fn set_ceil_reduction(md: &mut Self::MetaData, ceil_reduct: u8);

    /// Returns `true` if the block is currently linked inside a free list.
    ///
    /// This is a critical guard used to prevent **Double Free** vulnerabilities.
    fn is_linked(md: &Self::MetaData) -> bool;

    /// Marks whether the block is linked (`true`) or unlinked (`false`) in a free list.
    fn set_link(md: &mut Self::MetaData, state: bool);
}
