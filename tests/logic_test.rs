use bolero::AnySliceMutExt;
use buddy_core::{BuddyBase, BuddyError, IBuddyMdAdapter, IBuddyMetaData};

#[derive(Debug, Clone, Copy)]
pub struct TestMetaData {
    next: Option<u64>,
    last: Option<u64>,

    is_linked: bool,

    order: u8,
    ceil_reduct: u8,
}
impl TestMetaData {
    pub const fn new() -> Self {
        Self {
            next: None,
            last: None,
            is_linked: false,
            order: 0,
            ceil_reduct: 0,
        }
    }
}

impl IBuddyMetaData for TestMetaData {
    type MetaData = &'static mut TestMetaData;

    fn get_next(md: &Self::MetaData) -> Option<u64> {
        md.next
    }

    fn set_next(md: &mut Self::MetaData, n: Option<u64>) {
        md.next = n;
    }

    fn get_last(md: &Self::MetaData) -> Option<u64> {
        md.last
    }

    fn set_last(md: &mut Self::MetaData, n: Option<u64>) {
        md.last = n;
    }

    fn set_order(md: &mut Self::MetaData, order: u8) {
        md.order = order;
    }

    fn get_order(md: &Self::MetaData) -> u8 {
        md.order
    }

    fn get_ceil_reduction(md: &Self::MetaData) -> u8 {
        md.ceil_reduct
    }

    fn set_ceil_reduction(md: &mut Self::MetaData, ceil_reduct: u8) {
        md.ceil_reduct = ceil_reduct;
    }

    fn is_linked(md: &Self::MetaData) -> bool {
        md.is_linked
    }

    fn set_link(md: &mut Self::MetaData, state: bool) {
        md.is_linked = state;
    }
}

pub struct TestMetaDataHandler {}

static mut MDS: [TestMetaData; 4096 * 4] = [TestMetaData::new(); 4096 * 4];

impl IBuddyMdAdapter for TestMetaDataHandler {
    type Interface = TestMetaData;

    type MetaDataHandle = &'static mut TestMetaData;

    fn get_md(n: u64) -> Option<Self::MetaDataHandle> {
        unsafe {
            if n >= (4096 * 4) {
                None
            } else {
                Some(&mut MDS[n as usize])
            }
        }
    }
}

fn setup_initial_allocator() -> BuddyBase<TestMetaDataHandler, 13> {
    let mut alloc = BuddyBase::new();
    alloc.push_with_order(0, 12, 0).expect("push fail");
    alloc.push_with_order(4096, 11, 1).expect("push fail");
    alloc.push_with_order(6144, 11, 1).expect("push fail");
    alloc.push_with_order(8192, 12, 0).expect("push fail");
    alloc.push_with_order(12288, 10, 2).expect("push fail");
    alloc.push_with_order(13312, 10, 2).expect("push fail");
    alloc.push_with_order(14336, 10, 2).expect("push fail");
    alloc.push_with_order(15360, 9, 3).expect("push fail");
    alloc.push_with_order(15872, 9, 3).expect("push fail");
    alloc
}
#[test]
pub fn logic_test() {
    let ceil = buddy_core::allocate_ceil_reductor::<13>(1);
    assert_eq!(ceil, 11);

    //
    let mut alloc: BuddyBase<TestMetaDataHandler, 13> = setup_initial_allocator();

    let mut allocated_list = Vec::<u64>::new();

    while let Ok(f) = alloc.pop(0) {
        allocated_list.push(f);
    }

    allocated_list.shuffle();

    for i in allocated_list.drain(..) {
        alloc.push(i).expect("push faild");
    }

    let generator = bolero::produce::<Vec<(u8, u8)>>().with().len(10..100);
    let mut alloc_count = 0u64;
    let mut dealloc_count = 0u64;
    bolero::check!()
        .with_generator(generator)
        .for_each(|generator| {
            //
            //
            let mut allocated_list = Vec::<u64>::new();

            for &(op, raw_order) in generator {
                if op % 2 == 0 {
                    let safe_order = raw_order % 13;
                    if let Ok(addr) = alloc.pop(safe_order) {
                        allocated_list.push(addr);
                        alloc_count += 1;
                    }
                } else {
                    if let Some(addr) = allocated_list.pop() {
                        let _ = alloc.push(addr).expect("buddy internal faild");
                        dealloc_count += 1;
                    }
                }
            }

            for addr in allocated_list.drain(..) {
                alloc.push(addr).expect("buddy internal faild");
                dealloc_count += 1;
            }
        });

    assert_eq!(alloc_count, dealloc_count);

    alloc.pop(12).expect("data corrupted");
    alloc.pop(12).expect("data corrupted");
    assert_eq!(alloc.pop(12).unwrap_err(), BuddyError::NotFound);

    alloc.pop(11).expect("data corrupted");
    alloc.pop(11).expect("data corrupted");
    assert_eq!(alloc.pop(11).unwrap_err(), BuddyError::NotFound);

    alloc.pop(10).expect("data corrupted");
    alloc.pop(10).expect("data corrupted");
    alloc.pop(10).expect("data corrupted");
    assert_eq!(alloc.pop(10).unwrap_err(), BuddyError::NotFound);

    alloc.pop(9).expect("data corrupted");
    alloc.pop(9).expect("data corrupted");
    assert_eq!(alloc.pop(9).unwrap_err(), BuddyError::NotFound);
}
