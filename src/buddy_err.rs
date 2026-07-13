#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuddyError {
    DoubleFree,
    DataCorrupted,
    NotFound,
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
