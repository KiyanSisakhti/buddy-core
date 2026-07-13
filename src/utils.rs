#[inline]
pub fn buddy_lookup(n: u64, order: u8) -> u64 {
    let order_size: u64 = get_block_size_at_order(order) as u64;
    n ^ order_size
}

#[inline]
pub fn get_block_size_at_order(order: u8) -> usize {
    two_at_power(order) as usize
}

#[inline]
pub fn two_at_power(power: u8) -> u64 {
    1 << power
}

#[inline]
pub fn is_aligned_at_order(n: u64, order: u8) -> bool {
    const MASK: u64 = u64::MAX;

    let rm = !(MASK << order);

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
