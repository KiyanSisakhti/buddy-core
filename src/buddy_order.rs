use crate::{IBuddyMdAdapter, IBuddyMetaData};
use core::marker::PhantomData;

pub struct BuddyOrder<Addapter>
where
    Addapter: IBuddyMdAdapter,
{
    root: Option<u64>,
    pub order: u8,
    adapter: PhantomData<Addapter>,
}

impl<Adapter> BuddyOrder<Adapter>
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

    pub fn push(&mut self, n: u64) -> bool {
        let md_res = Adapter::get_md(n);

        match md_res {
            Some(mut md) => {
                if Adapter::Interface::is_linked(&md) {
                    return false;
                }

                Adapter::Interface::set_link(&mut md, true);
                Adapter::Interface::set_order(&mut md, self.order);

                if let Some(root) = self.root {
                    let mut root_md = Adapter::get_md(root).expect("system data corupted");

                    Adapter::Interface::set_last(&mut root_md, Some(n));

                    Adapter::Interface::set_last(&mut md, None);
                    Adapter::Interface::set_next(&mut md, Some(root));

                    self.root = Some(n);

                    true
                } else {
                    Adapter::Interface::set_last(&mut md, None);
                    Adapter::Interface::set_next(&mut md, None);
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

        let mut md = Adapter::get_md(root).expect("system data corupted");

        let Some(next) = Adapter::Interface::get_next(&md) else {
            self.root = None;

            Adapter::Interface::set_next(&mut md, None);
            Adapter::Interface::set_last(&mut md, None);
            Adapter::Interface::set_link(&mut md, false);

            return Some(root);
        };

        let mut next_md = Adapter::get_md(next).expect("system data corupted");

        Adapter::Interface::set_last(&mut next_md, None);

        Adapter::Interface::set_last(&mut md, None);
        Adapter::Interface::set_next(&mut md, None);
        Adapter::Interface::set_link(&mut md, false);

        self.root = Some(next);
        Some(root)
    }

    fn try_unlink_at_order(n: u64, order: u8) -> bool {
        let Some(mut md) = Adapter::get_md(n) else {
            return false;
        };
        if !Adapter::Interface::is_linked(&md) || Adapter::Interface::get_order(&md) != order {
            return false;
        }

        let last = Adapter::Interface::get_last(&md);
        let next = Adapter::Interface::get_next(&md);

        if let Some(lst) = last {
            let mut lst_md = Adapter::get_md(lst).expect("system data corupted");

            Adapter::Interface::set_next(&mut lst_md, next);
        }
        if let Some(nxt) = next {
            let mut nxt_md = Adapter::get_md(nxt).expect("system data corupted");

            Adapter::Interface::set_last(&mut nxt_md, last);
        }

        Adapter::Interface::set_next(&mut md, None);
        Adapter::Interface::set_last(&mut md, None);
        Adapter::Interface::set_link(&mut md, false);

        return true;
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
