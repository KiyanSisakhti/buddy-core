use crate::{
    IBuddyMdAdapter, IBuddyMetaData,
    buddy_err::BuddyError,
    buddy_order::BuddyOrder,
    utils::{buddy_lookup, is_aligned_at_order},
};

pub struct BuddyBase<Adapter, const MAX_ORDER: usize = 2>
where
    Adapter: IBuddyMdAdapter,
{
    orders: [BuddyOrder<MAX_ORDER, Adapter>; MAX_ORDER],
}

impl<Adapter, const MAX_ORDER: usize> BuddyBase<Adapter, MAX_ORDER>
where
    Adapter: IBuddyMdAdapter,
{
    const MAX_CHECK: () = {
        assert!(MAX_ORDER <= 31);
    };

    pub fn new() -> Self {
        let _ = Self::MAX_CHECK;
        Self {
            orders: core::array::from_fn(|d| BuddyOrder::new(d as u8)),
        }
    }

    pub fn allocate_ceil_reductor(max_order: u8) -> u8 {
        (MAX_ORDER as u8 - 1) - max_order
    }

    pub fn push_with_order(
        &mut self,
        n: u64,
        order: u8,
        ceil_reduct: u8,
    ) -> Result<(), BuddyError> {
        if !is_aligned_at_order(n, order) {
            return Err(BuddyError::AlignmentMismatch);
        }

        let Some(mut md) = Adapter::get_md(n) else {
            return Err(BuddyError::DataCorrupted);
        };
        Adapter::Interface::set_ceil_reduction(&mut md, ceil_reduct);
        Adapter::Interface::set_order(&mut md, order);

        self.push(n)
    }

    pub fn push(&mut self, n: u64) -> Result<(), BuddyError> {
        let Some(md) = Adapter::get_md(n) else {
            return Err(BuddyError::DataCorrupted);
        };

        let ord = Adapter::Interface::get_order(&md);
        let ceil_reduct = Adapter::Interface::get_ceil_reduction(&md);

        if !is_aligned_at_order(n, ord) {
            return Err(BuddyError::DataCorrupted);
        }

        let buddy_n = buddy_lookup(n, ord);
        if let Some(buddy_md) = Adapter::get_md(buddy_n) {
            if Adapter::Interface::get_order(&buddy_md) > ord {
                return Err(BuddyError::DoubleFree);
            }
        };

        self.insert_fix(n, ord, ceil_reduct)
    }
    fn insert_fix(&mut self, n: u64, order: u8, ceil_reductor: u8) -> Result<(), BuddyError> {
        let bd_ord = &mut self.orders[order as usize];
        let ceiled_max_ord = (MAX_ORDER as u8) - ceil_reductor;
        if (order + 1) >= ceiled_max_ord {
            return bd_ord.push(n);
        };

        let buddy_n = buddy_lookup(n, order);

        if let Err(err) = bd_ord.try_remove_at_order(buddy_n, order + 1) {
            match err {
                BuddyError::NotFound => {
                    return bd_ord.push(n);
                }
                _ => return Err(err),
            }
        }

        let min = n.min(buddy_n);
        self.insert_fix(min, order + 1, ceil_reductor)
    }

    pub fn pop(&mut self, order: u8) -> Result<u64, BuddyError> {
        let bd_ord = &mut self.orders[order as usize];

        let num = match bd_ord.pop(order) {
            Ok(n) => n,
            Err(BuddyError::NotFound) => self.buddy_emission(order)?,
            Err(err) => return Err(err),
        };

        let mut md = Adapter::get_md(num).ok_or(BuddyError::DataCorrupted)?;
        Adapter::Interface::set_order(&mut md, order);

        Ok(num)
    }

    fn buddy_emission(&mut self, targ: u8) -> Result<u64, BuddyError> {
        let nxt_trg = targ + 1;
        if nxt_trg >= MAX_ORDER as u8 {
            return Err(BuddyError::NotFound);
        }

        let n = match self.orders[nxt_trg as usize].pop(targ) {
            Ok(n) => n,
            Err(BuddyError::NotFound) => self.buddy_emission(nxt_trg)?,
            Err(err) => return Err(err),
        };

        let bd = buddy_lookup(n, targ);

        self.orders[targ as usize].push(bd)?;

        Ok(n)
    }

    pub fn dump(&self) {
        for t in &self.orders {
            println!("\nl {}:", t.order);
            t.dump();
        }
    }
}
