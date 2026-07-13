/// Errors that can occur during buddy allocator operations.
///
/// These errors help ensure memory safety, catch double-free bugs,
/// and detect structural corruptions inside the allocator state.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuddyError {
    /// Discovered an attempt to free a block that is already marked as free or linked.
    DoubleFree,

    /// The metadata for the requested block index is either missing,
    /// altered out of bounds, or structurally corrupted.
    DataCorrupted,

    /// No free blocks are available at the requested target order,
    /// and higher orders cannot be split further.
    NotFound,

    /// The provided block address is not aligned properly with respect
    /// to its allocation or buddy order.
    AlignmentMismatch,
}

impl core::fmt::Display for BuddyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::DoubleFree => write!(f, "Page double free detected"),
            Self::DataCorrupted => write!(f, "Buddy metadata is corrupted or missing"),
            Self::NotFound => write!(f, "No free block found at the requested order"),
            Self::AlignmentMismatch => write!(f, "block not aligned"),
        }
    }
}

impl core::error::Error for BuddyError {}
