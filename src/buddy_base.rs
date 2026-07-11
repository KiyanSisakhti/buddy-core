use crate::{IBuddyMdAdapter, IBuddyMetaData, buddy_order::BuddyOrder};

pub struct BuddyBase<BMD, const MAX_ORDER: usize = 2>
where
    BMD: IBuddyMdAdapter,
{
    orders: [BuddyOrder<BMD>; MAX_ORDER],
}

impl<BMD, const MAX_ORDER: usize> BuddyBase<BMD, MAX_ORDER>
where
    BMD: IBuddyMdAdapter,
{
    pub fn new() -> Self {
        Self {
            orders: core::array::from_fn(|d| BuddyOrder::new(d as u8)),
        }
    }

    pub fn put(&mut self, n: u64) -> bool {
        let Some(md) = BMD::ref_md(n) else {
            return false;
        };

        let ord = md.get_order() as usize;

        let r = &mut self.orders[ord];

        r.push(n);

        true
    }

    pub fn dump(&mut self, order: usize) {
        let r = &mut self.orders[order];

        r.dump();
    }
}
