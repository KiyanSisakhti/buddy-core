use crate::{IBuddyMdAdapter, IBuddyMetaData};
use core::marker::PhantomData;

pub struct BuddyOrder<T>
where
    T: IBuddyMdAdapter,
{
    root: Option<u64>,
    order: u8,
    addapter: PhantomData<T>,
}

impl<T> BuddyOrder<T>
where
    T: IBuddyMdAdapter,
{
    pub fn new(order: u8) -> Self {
        Self {
            root: None,
            addapter: PhantomData,
            order,
        }
    }

    pub fn push(&mut self, n: u64) -> bool {
        let md_res = T::mut_md(n);

        match md_res {
            Some(md) => {
                if let Some(root) = self.root {
                    let root_md = T::mut_md(root).expect("system data corupted");
                    let last = root_md.get_last();
                    let last_md = T::mut_md(last).expect("system data corupted");

                    last_md.set_next(n);
                    md.set_last(last);
                    md.set_next(root);
                    md.set_link(true);
                    root_md.set_last(n);

                    true
                } else {
                    md.set_link(true);
                    md.set_last(n);
                    md.set_next(n);
                    self.root = Some(n);
                    true
                }
            }
            None => false,
        }
    }

    pub fn dump(&self) {
        let mut n = self.root;

        while let Some(s) = n {
            println!("{:?} - ", s);

            n = T::ref_md(s).map(|d| d.get_next());
        }
    }
}
