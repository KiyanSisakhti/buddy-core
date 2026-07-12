use crate::{
    IBuddyMdAdapter, IBuddyMetaData,
    buddy_order::BuddyOrder,
    utils::{buddy_lookup, is_aligned_at_order},
};

pub struct BuddyBase<Adapter, const MAX_ORDER: usize = 2>
where
    Adapter: IBuddyMdAdapter,
{
    orders: [BuddyOrder<Adapter>; MAX_ORDER],
}

impl<Adapter, const MAX_ORDER: usize> BuddyBase<Adapter, MAX_ORDER>
where
    Adapter: IBuddyMdAdapter,
{
    pub fn new() -> Self {
        Self {
            orders: core::array::from_fn(|d| BuddyOrder::new(d as u8)),
        }
    }

    pub fn allocate_ceil_reductor(max_order: u8) -> u8 {
        (MAX_ORDER as u8 - 1) - max_order
    }

    pub fn push(&mut self, n: u64) -> bool {
        let Some(md) = Adapter::get_md(n) else {
            return false;
        };

        let ord = Adapter::Interface::get_order(&md);
        let ceil_reduct = Adapter::Interface::get_ceil_reduction(&md);

        if !is_aligned_at_order(n, ord) {
            return false;
        }

        self.insert_fix(n, ord, ceil_reduct)
    }
    fn insert_fix(&mut self, n: u64, order: u8, ceil_reductor: u8) -> bool {
        let bd_ord = &mut self.orders[order as usize];
        let ceiled_max_ord = (MAX_ORDER as u8) - ceil_reductor;
        if (order + 1) >= ceiled_max_ord {
            return bd_ord.push(n);
        };

        let buddy_n = buddy_lookup(n, order);

        if !bd_ord.try_remove_at_order(buddy_n) {
            return bd_ord.push(n);
        }

        let min = n.min(buddy_n);
        self.insert_fix(min, order + 1, ceil_reductor)
    }

    pub fn pop(&mut self, order: u8) -> Option<u64> {
        let bd_ord = &mut self.orders[order as usize];

        let num = match bd_ord.pop() {
            Some(n) => n,
            None => self.buddy_emission(order)?,
        };

        if let Some(mut md) = Adapter::get_md(num) {
            Adapter::Interface::set_order(&mut md, order);
        }
        Some(num)
    }

    fn buddy_emission(&mut self, targ: u8) -> Option<u64> {
        let nxt_trg = targ + 1;
        if nxt_trg >= MAX_ORDER as u8 {
            return None;
        }

        let n = self.orders[nxt_trg as usize]
            .pop()
            .or_else(|| self.buddy_emission(nxt_trg))?;

        let bd = buddy_lookup(n, targ);

        self.orders[targ as usize].push(bd);

        Some(n)
    }

    pub fn dump(&self) {
        for t in &self.orders {
            println!("\nl {}:", t.order);
            t.dump();
        }
    }
}
