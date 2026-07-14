/// Manages a single order layer (level) inside a Buddy Allocator ecosystem.
///
/// Each `BuddyOrder` is responsible for keeping track of a doubly-linked list of free memory
/// blocks that belong strictly to its assigned allocation level (`order`). It coordinates
/// micro-operations like head-insertions (`push`), fast retrievals (`pop`), and middle-node
/// extractions (`try_unlink_at_order`) when buddies are merging or splitting.
///
/// # Type Parameters
/// * `ORDER_COUNT` - The compile-time limit of the maximum allowable orders in the entire allocator.
/// * `Adapter` - The hardware/storage translation abstraction implementing [`IBuddyMdAdapter`].
///
///
use crate::{IBuddyMdAdapter, IBuddyMetaData, buddy_err::BuddyError};
use core::marker::PhantomData;

pub struct BuddyOrder<const ORDER_COUNT: usize, Adapter>
where
    Adapter: IBuddyMdAdapter,
{
    /// The index of the head block currently pointing to the start of this order's free list.
    root: Option<u64>,

    /// The specific numerical order level managed by this instance (e.g., 0, 1, 2...).
    pub order: u8,

    /// A zero-sized marker to anchor the generic `Adapter` compile-time type specification.
    adapter: PhantomData<Adapter>,
}

impl<const ORDER_COUNT: usize, Adapter> BuddyOrder<ORDER_COUNT, Adapter>
where
    Adapter: IBuddyMdAdapter,
{
    /// Creates a new, unlinked container for a specific order level.
    pub fn new(order: u8) -> Self {
        Self {
            root: None,
            adapter: PhantomData,
            order,
        }
    }

    /// Inserts a free memory block at the head of this order's doubly-linked list.
    ///
    /// # Errors
    /// * [`BuddyError::DoubleFree`] - If the block's metadata indicates it is already linked somewhere.
    /// * [`BuddyError::DataCorrupted`] - If active pointers inside the root or node metadata fail lookup bounds.
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

    /// Dispatches the removal of a specific block depending on whether it sits at the root or deep in the list.
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

    /// Evaluates if a block is mathematically allowed to exist or merge into the target ceiling.
    ///
    /// Uses branchless-friendly linear arithmetic to confirm whether the current order plus any ceiling reductions
    /// remains strictly less than the total system max orders limit ($O(1)$ complexity).
    #[inline(always)]
    fn can_be_at_order(
        md: &<Adapter as IBuddyMdAdapter>::MetaDataHandle,
        target_order: u8,
    ) -> bool {
        let ceil_reduct = Adapter::Interface::get_ceil_reduction(md);
        (target_order as usize) + (ceil_reduct as usize) < ORDER_COUNT
    }

    /// Pops the root (head) block off this order layer, promoting it to the next higher order level state.
    ///
    /// This is typically performed when two buddies are merging upwards, or when a block is allocated out of this layer.
    ///
    /// # Errors
    /// * [`BuddyError::NotFound`] - If the list is empty, or if the block violates ceiling limits (`can_be_at_order`).
    /// * [`BuddyError::DataCorrupted`] - If pointers in the next cascading nodes are invalid.
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

    /// Extracts a specific node from the middle of the doubly linked list, repairing surrounding pointers.
    ///
    /// This method performs an absolute logical unlink. It handles sewing the `next` pointer of the previous
    /// node directly to the `last` pointer of the following node, cleanly lifting the extracted block away.
    /// Once unlinked, the block's internal state tracking parameters are elevated to `order + 1`.
    ///
    /// # Errors
    /// * [`BuddyError::NotFound`] - If the block is unmapped, fails validation bounds, or is already detached.
    /// * [`BuddyError::DataCorrupted`] - If neighboring references in the linked sequence fail metadata lookup.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // This is a conceptual manual scenario demonstrating how layers are isolated and asserted.
    /// // It is marked as `ignore` so that standard automated cargo runners skip testing this unlinked context.
    /// use crate::{BuddyOrder, BuddyError};
    ///
    /// fn manual_unlink_simulation() -> Result<(), BuddyError> {
    ///     let mut layer = BuddyOrder::<13, MyCustomAdapter>::new(0);
    ///
    ///     // Inject three consecutive physical block indices
    ///     layer.push(100)?;
    ///     layer.push(200)?;
    ///     layer.push(300)?;
    ///
    ///     // Force extraction of the middle block (200) during a buddy allocation merge event
    ///     layer.try_remove_at_order(200, 0)?;
    ///
    ///     // Assert structural changes on the detached nodes manually
    ///     Ok(())
    /// }
    /// ```
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

        Ok(())
    }

    // Using on debug
    // pub fn dump(&self) {
    //     let mut root: Option<u64> = self.root;
    //     while let Some(rt) = root {
    //         print!("-{}-", rt);

    //         let fg = Adapter::get_md(rt).unwrap();
    //         root = Adapter::Interface::get_next(&fg);
    //     }
    // }
}
