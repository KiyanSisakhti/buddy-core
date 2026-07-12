use crate::{IBuddyMdAdapter, IBuddyMetaData};
use core::marker::PhantomData;

pub struct BuddyOrder<T>
where
    T: IBuddyMdAdapter,
{
    root: Option<u64>,
    pub order: u8,
    adapter: PhantomData<T>,
}

impl<T> BuddyOrder<T>
where
    T: IBuddyMdAdapter,
{
    pub fn new(order: u8) -> Self {
        Self {
            root: None,
            adapter: PhantomData,
            order,
        }
    }

    pub fn push(&mut self, n: u64) -> bool {
        let md_res = T::mut_md(n);

        match md_res {
            Some(md) => {
                if md.is_linked() {
                    return false;
                }
                //
                md.set_link(true);
                md.set_order(self.order);

                if let Some(root) = self.root {
                    let root_md = T::mut_md(root).expect("system data corupted");
                    root_md.set_last(Some(n));

                    md.set_last(None);
                    md.set_next(Some(root));

                    self.root = Some(n);

                    true
                } else {
                    md.set_last(None);
                    md.set_next(None);
                    self.root = Some(n);

                    true
                }
            }
            None => false,
        }
    }

    pub fn try_remove_at_order(&mut self, n: u64) -> bool {
        if let Some(root) = self.root {
            if root == n {
                self.pop().is_some()
            } else {
                Self::try_unlink_at_order(n, self.order)
            }
        } else {
            false
        }
    }

    pub fn pop(&mut self) -> Option<u64> {
        let root = self.root?;

        let md = T::mut_md(root).expect("system data corupted");

        let Some(next) = md.get_next() else {
            self.root = None;

            md.set_next(None);
            md.set_last(None);
            md.set_link(false);

            return Some(root);
        };

        let next_md = T::mut_md(next).expect("system data corupted");

        next_md.set_last(None);

        md.set_last(None);
        md.set_next(None);
        md.set_link(false);

        self.root = Some(next);
        Some(root)
    }

    fn try_unlink_at_order(n: u64, order: u8) -> bool {
        let Some(md) = T::mut_md(n) else {
            return false;
        };
        if !md.is_linked() || md.get_order() != order {
            return false;
        }

        let last = md.get_last();
        let next = md.get_next();

        if let Some(lst) = last {
            let lst_md = T::mut_md(lst).expect("system data corupted");
            lst_md.set_next(next);
        }
        if let Some(nxt) = next {
            let nxt_md = T::mut_md(nxt).expect("system data corupted");
            nxt_md.set_last(last);
        }

        md.set_next(None);
        md.set_last(None);
        md.set_link(false);

        return true;
    }

    pub fn dump(&self) {
        let mut root: Option<u64> = self.root;
        while let Some(rt) = root {
            print!("-{}-", rt);

            root = T::mut_md(rt).unwrap().get_next();
        }
    }
}
