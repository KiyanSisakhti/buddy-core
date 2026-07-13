/// A hardware and storage abstraction layer for the Buddy Allocator's metadata.
///
/// This trait acts as a bridge between the core allocator logic and the actual physical
/// or virtual storage where the page metadata resides. By implementing this trait,
/// the allocator becomes decoupled from the underlying system architecture, allowing it
/// to run on bare-metal embedded systems, operating system kernels, or standard user-space test environments.
///
/// # Why This Architecture Matters
/// The buddy allocator needs to read and write states (like order, links, and ceilings)
/// for billions of memory blocks very quickly. Instead of hardcoding *how* and *where*
/// this metadata is looked up, the allocator delegates that responsibility to this adapter.
///
use crate::IBuddyMetaData;

pub trait IBuddyMdAdapter {
    /// Specifies the concrete metadata behavior type.
    ///
    /// This type must implement the [`IBuddyMetaData`] trait, ensuring that whoever provides
    /// the metadata also provides the getter and setter interfaces to manipulate its internal state.
    /// The associated `MetaData` type inside the interface must strictly match our [`Self::MetaDataHandle`].
    type Interface: IBuddyMetaData<MetaData = Self::MetaDataHandle>;

    /// Defines the container or pointer type used to pass and manipulate metadata references.
    ///
    /// In standard environments, this is typically a mutable reference like `&'static mut PageMetaData`.
    /// In more complex architectures, it could be a smart pointer or a custom index wrapper
    /// tailored to satisfy Rust's strict borrowing and ownership rules.
    type MetaDataHandle;

    ///
    ///
    /// Resolves a raw physical block index or memory address into a safely accessible metadata handle.
    ///
    /// This is the most critical and performance-sensitive function in the entire allocator ecosystem.
    /// It maps a raw `u64` reference identifier (like a page frame number or block index) to its
    /// corresponding metadata entry.
    ///
    /// # Performance Expectations
    /// Since this method is invoked inside the innermost loops of the allocator's critical hot path
    /// (during rapid allocations, page splits, and buddy merges), its implementation must be
    /// highly optimized. It should ideally boil down to a simple, branchless pointer arithmetic
    /// calculation or a direct array index lookup ($O(1)$ complexity) to maximize CPU L1 cache hits.
    ///
    /// # Returns
    /// * `Some(MetaDataHandle)` - A valid, ready-to-use handle containing the target block's state.
    /// * `None` - If the requested index is out of bounds, unmapped, or structurally invalid.
    fn get_md(n: u64) -> Option<Self::MetaDataHandle>;
}
