//! # Utility Functions for Bitwise Buddy Math
//!
//! This module provides the core bitwise and mathematical foundations required by the
//! buddy allocator. It handles fast power-of-two calculations, block size lookups,
//! alignment verification, and buddy address computation.
//!
//! All functions are decorated with `#[inline]` to ensure the compiler can optimize away
//! any function call overhead, generating direct assembly instructions.

/// Calculates the companion "buddy" address of a given block address at a specific order.
///
/// In a binary buddy allocator, a block and its buddy differ by exactly one bit—the bit
/// representing the size of the block at that order. XORing the block address with the
/// order size flips this bit, instantly revealing the buddy's address.
///
/// # Mathematical Principle
/// For a block at address $n$ and order $o$, its buddy is:
/// $$buddy = n \oplus (1 \ll o)$$
///
/// # Examples
/// If we have a block at address `0` at Order 0 (size 1):
/// * `0 ^ (1 << 0) => 0 ^ 1 = 1` (Page 0 and Page 1 are buddies).
/// * If we have a block at address `0` at Order 1 (size 2):
/// * `0 ^ (1 << 1) => 0 ^ 2 = 2` (Block 0-1 and Block 2-3 are buddies).
#[inline]
pub fn buddy_lookup(n: u64, order: u8) -> u64 {
    let order_size: u64 = get_block_size_at_order(order) as u64;
    n ^ order_size
}

/// Computes the span/size of a single memory block at the specified order level.
///
/// Since the buddy system works in powers of two, the size at any given order
/// is computed as $2^{\text{order}}$.
#[inline]
pub fn get_block_size_at_order(order: u8) -> usize {
    two_at_power(order) as usize
}

/// Calculates $2^{\text{power}}$ using an efficient left-shift operation.
///
/// This is mathematically equivalent to $1 \ll \text{power}$.
///
/// # Safety / Limits
/// The caller must ensure that `power` does not exceed 63, otherwise this operation
/// will trigger an overflow / panic in debug mode, or wrap-around in release mode.
#[inline]
pub fn two_at_power(power: u8) -> u64 {
    1 << power
}

/// Determines whether a physical address/index `n` is aligned to the boundary of a specific order.
///
/// For a block to be valid at `order`, its address must be a multiple of the block's size
/// ($2^{\text{order}}$). This means the lowest `order` bits of the address must all be zeros.
///
/// # Bitwise Masking Mechanism
/// 1. We start with a full mask of 1s: `u64::MAX` (`0xFFFFFFFFFFFFFFFF`).
/// 2. Shifting it left by `order` creates a mask like `0xFFFFFFFFFFFFFFF0` (for order 4).
/// 3. Inverting this mask with `!` gives `0x000000000000000F`, isolating the lower offset bits.
/// 4. Performing a bitwise AND (`n & rm`) checks if any of these offset bits are set.
///    If the result is `0`, the address is perfectly aligned.
#[inline]
pub fn is_aligned_at_order(n: u64, order: u8) -> bool {
    const MASK: u64 = u64::MAX;

    // Create a bitmask for the lower 'order' bits.
    // e.g., for order = 3, !(0xFFFFFFFFFFFFFFF8) => 0x0000000000000007
    let rm = !(MASK << order);

    // If no lower bits are set, the address is aligned to 2^order.
    (n & rm) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_two_at_power() {
        assert_eq!(two_at_power(0), 1);
        assert_eq!(two_at_power(3), 8);
        assert_eq!(two_at_power(6), 64);
    }

    #[test]
    fn test_get_block_size_at_order() {
        assert_eq!(get_block_size_at_order(0), 1);
        assert_eq!(get_block_size_at_order(4), 16);
    }

    #[test]
    fn test_buddy_lookup() {
        // Order 0 (block size 1): buddy of 0 is 1, buddy of 1 is 0
        assert_eq!(buddy_lookup(0, 0), 1);
        assert_eq!(buddy_lookup(1, 0), 0);

        // Order 2 (block size 4): buddies are pairs (0,4), (1,5), (2,6), (3,7), etc.
        assert_eq!(buddy_lookup(0, 2), 4);
        assert_eq!(buddy_lookup(4, 2), 0);
        assert_eq!(buddy_lookup(5, 2), 1);
    }

    #[test]
    fn test_is_aligned_at_order() {
        // Any number is aligned at order 0
        assert!(is_aligned_at_order(0, 0));
        assert!(is_aligned_at_order(7, 0));

        // Order 3 requires alignment to 8 bytes (multiples of 8)
        assert!(is_aligned_at_order(0, 3));
        assert!(is_aligned_at_order(8, 3));
        assert!(is_aligned_at_order(16, 3));

        // Non-multiples of 8 should fail alignment at order 3
        assert!(!is_aligned_at_order(4, 3));
        assert!(!is_aligned_at_order(9, 3));
    }
}
