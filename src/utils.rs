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

pub fn is_aligned_at_order(n: u64, order: u8) -> bool {
    const MASK: u64 = u64::MAX;

    let rm = !(MASK << order);

    (n & rm) == 0
}
