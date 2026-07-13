use buddy_core::{BuddyBase, IBuddyMdAdapter, IBuddyMetaData};

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

static mut MDS: [TestMetaData; 4096 * 2] = [TestMetaData::new(); 4096 * 2];

impl IBuddyMdAdapter for TestMetaDataHandler {
    type Interface = TestMetaData;

    type MetaDataHandle = &'static mut TestMetaData;

    fn get_md(n: u64) -> Option<Self::MetaDataHandle> {
        unsafe {
            if n >= (4096 * 2) {
                None
            } else {
                Some(&mut MDS[n as usize])
            }
        }
    }
}

#[test]
pub fn logic_test() {
    //
    let mut alloc: BuddyBase<TestMetaDataHandler, 13> = BuddyBase::new();

    alloc.dump();
    println!("\n\n");

    alloc.push_with_order(2048, 11, 1).unwrap();
    alloc.push_with_order(0, 11, 0).unwrap();

    // alloc.push(0).unwrap();
    // alloc.push(1).unwrap();
    // alloc.push(2).unwrap();
    // alloc.push(3).unwrap();
    // alloc.push(4).unwrap();
    // alloc.push(5).unwrap();
    // alloc.push(6).unwrap();
    // alloc.push(7).unwrap();
    // alloc.push(8).unwrap();
    // alloc.push(9).unwrap();
    // alloc.push(10).unwrap();
    // alloc.push(11).unwrap();
    // alloc.push(12).unwrap();
    // alloc.push(13).unwrap();
    // alloc.push(14).unwrap();
    // alloc.push(15).unwrap();
    // alloc.push(16).unwrap();
    // alloc.push(17).unwrap();
    // alloc.push(18).unwrap();
    // alloc.push(19).unwrap();
    // alloc.push(20).unwrap();
    // alloc.push(21).unwrap();
    // alloc.push(22).unwrap();
    // alloc.push(23).unwrap();
    // alloc.push(24).unwrap();
    // alloc.push(25).unwrap();
    // alloc.push(26).unwrap();
    // alloc.push(27).unwrap();
    // alloc.push(28).unwrap();
    // alloc.push(29).unwrap();
    // alloc.push(30).unwrap();
    // alloc.push(31).unwrap();

    // alloc.dump();
    // println!("\n\n");
    // let rfv = alloc.pop(0).unwrap();
    // let rfv2 = alloc.pop(0).unwrap();
    // let f = alloc.pop(2).unwrap();
    // let f1 = alloc.pop(2).unwrap();
    // let f2 = alloc.pop(0).unwrap();
    // let f3 = alloc.pop(0).unwrap();
    // let f4 = alloc.pop(1).unwrap();
    // let f5 = alloc.pop(1).unwrap();
    // let f6 = alloc.pop(3).unwrap();
    // let f7 = alloc.pop(1).unwrap();
    // let f8 = alloc.pop(0).unwrap();
    // let f9 = alloc.pop(1).unwrap();
    // let f10 = alloc.pop(0).unwrap();

    // alloc.dump();

    // println!("\n\n\n\n");

    // alloc.push(rfv).unwrap();
    // alloc.push(rfv2).unwrap();
    // alloc.push(f).unwrap();
    // alloc.push(f1).unwrap();
    // alloc.push(f2).unwrap();
    // alloc.push(f3).unwrap();
    // alloc.push(f4).unwrap();
    // alloc.push(f5).unwrap();
    // alloc.push(f6).unwrap();
    // alloc.push(f7).unwrap();
    // alloc.push(f8).unwrap();
    // alloc.push(f9).unwrap();
    // alloc.push(f10).unwrap();

    // alloc.push(f3).expect_err("msg");
    // alloc.push(f4).expect_err("msg");
    // alloc.push(f1).expect_err("msg");
    // alloc.push(f5).expect_err("msg");
    // alloc.push(f6).expect_err("msg");
    // alloc.push(f7).expect_err("msg");
    // alloc.push(f2).expect_err("msg");
    // alloc.push(f).expect_err("msg");
    // alloc.push(rfv).expect_err("msg");
    // alloc.push(rfv2).expect_err("msg");
    // alloc.push(f8).expect_err("msg");
    // alloc.push(f10).expect_err("msg");
    // alloc.push(f9).expect_err("msg");

    // alloc.dump();

    // println!("\n\n");

    // println!("-{}-", buddy_lookup(5, 0));
    // println!("-{}-", buddy_lookup(5, 0));
    // println!("-{}-", buddy_lookup(5, 0));
    // println!("-{}-", BuddyBase::<TestDataBox>::rm_(1, 0));
    // println!("{}", is_aligned_at_order(2, 2))
    // bb.dump(1);

    // println!("----- lo c ------{f:?}");
    // println!("{:#?}", unsafe { MDS });

    // println!("----- lo c ------");
    // println!("----- lo c ------");
    // println!("----- lo c ------");
    // println!("----- lo c ------");
}
