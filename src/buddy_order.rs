use crate::{IBuddyMdAdapter, IBuddyMetaData, buddy_err::BuddyError};
use core::marker::PhantomData;

pub struct BuddyOrder<const MAX_ORDERS: usize, Adapter>
where
    Adapter: IBuddyMdAdapter,
{
    root: Option<u64>,
    pub order: u8,
    adapter: PhantomData<Adapter>,
}

impl<const MAX_ORDERS: usize, Adapter> BuddyOrder<MAX_ORDERS, Adapter>
where
    Adapter: IBuddyMdAdapter,
{
    pub fn new(order: u8) -> Self {
        Self {
            root: None,
            adapter: PhantomData,
            order,
        }
    }
    #[inline]
    pub fn push(&mut self, n: u64) -> Result<(), BuddyError> {
        let md_res = Adapter::get_md(n);

        match md_res {
            Some(mut md) => {
                if Adapter::Interface::is_linked(&md) {
                    return Err(BuddyError::DoubleFree);
                }

                Adapter::Interface::set_link(&mut md, true);
                Adapter::Interface::set_order(&mut md, self.order);

                if let Some(root) = self.root {
                    let mut root_md = Adapter::get_md(root).ok_or(BuddyError::DataCorrupted)?;

                    Adapter::Interface::set_last(&mut root_md, Some(n));

                    Adapter::Interface::set_last(&mut md, None);
                    Adapter::Interface::set_next(&mut md, Some(root));

                    self.root = Some(n);

                    Ok(())
                } else {
                    Adapter::Interface::set_last(&mut md, None);
                    Adapter::Interface::set_next(&mut md, None);
                    self.root = Some(n);

                    Ok(())
                }
            }
            None => Err(BuddyError::DataCorrupted),
        }
    }

    #[inline]
    pub fn try_remove_at_order(&mut self, n: u64, target_order: u8) -> Result<(), BuddyError> {
        if let Some(root) = self.root {
            if root == n {
                let _ = self.pop(target_order)?;
                Ok(())
            } else {
                Self::try_unlink_at_order(n, self.order, target_order)
            }
        } else {
            Err(BuddyError::NotFound)
        }
    }

    #[inline(always)]
    fn can_be_at_order(
        md: &<Adapter as IBuddyMdAdapter>::MetaDataHandle,
        target_order: u8,
    ) -> bool {
        let ceil_reduct = Adapter::Interface::get_ceil_reduction(md);
        (target_order as usize) + (ceil_reduct as usize) < MAX_ORDERS
    }

    #[inline]
    pub fn pop(&mut self, target_order: u8) -> Result<u64, BuddyError> {
        let root = self.root.ok_or(BuddyError::NotFound)?;

        let mut md = Adapter::get_md(root).ok_or(BuddyError::DataCorrupted)?;

        if !Self::can_be_at_order(&md, target_order) {
            return Err(BuddyError::NotFound);
        }

        let nxt_md = Adapter::Interface::get_next(&md);

        Adapter::Interface::set_next(&mut md, None);
        Adapter::Interface::set_last(&mut md, None);
        Adapter::Interface::set_link(&mut md, false);
        Adapter::Interface::set_order(&mut md, self.order + 1);

        let Some(next) = nxt_md else {
            self.root = None;

            return Ok(root);
        };

        let mut next_md = Adapter::get_md(next).ok_or(BuddyError::DataCorrupted)?;
        Adapter::Interface::set_last(&mut next_md, None);

        self.root = Some(next);
        Ok(root)
    }

    fn try_unlink_at_order(n: u64, order: u8, target_order: u8) -> Result<(), BuddyError> {
        let Some(mut md) = Adapter::get_md(n) else {
            return Err(BuddyError::NotFound);
        };

        if !Self::can_be_at_order(&md, target_order)
            || !Adapter::Interface::is_linked(&md)
            || Adapter::Interface::get_order(&md) != order
        {
            return Err(BuddyError::NotFound);
        }

        let last = Adapter::Interface::get_last(&md);
        let next = Adapter::Interface::get_next(&md);

        if let Some(lst) = last {
            let mut lst_md = Adapter::get_md(lst).ok_or(BuddyError::DataCorrupted)?;

            Adapter::Interface::set_next(&mut lst_md, next);
        }
        if let Some(nxt) = next {
            let mut nxt_md = Adapter::get_md(nxt).ok_or(BuddyError::DataCorrupted)?;

            Adapter::Interface::set_last(&mut nxt_md, last);
        }

        Adapter::Interface::set_next(&mut md, None);
        Adapter::Interface::set_last(&mut md, None);
        Adapter::Interface::set_link(&mut md, false);
        Adapter::Interface::set_order(&mut md, order + 1);

        return Ok(());
    }

    pub fn dump(&self) {
        let mut root: Option<u64> = self.root;
        while let Some(rt) = root {
            print!("-{}-", rt);

            let fg = Adapter::get_md(rt).unwrap();
            root = Adapter::Interface::get_next(&fg);
        }
    }
}
