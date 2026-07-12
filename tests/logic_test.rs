use buddy_allocation_adapter::{BuddyBase, IBuddyMdAdapter, IBuddyMetaData};

#[derive(Debug, Clone, Copy)]
pub struct TestMd {
    next: Option<u64>,
    last: Option<u64>,

    is_linked: bool,

    order: u8,
    ceil_reduct: u8,
}
impl TestMd {
    pub const fn new() -> Self {
        Self {
            next: None,
            last: None,
            is_linked: false,
            order: 0,
            ceil_reduct: 1,
        }
    }
}

impl IBuddyMetaData for TestMd {
    type MetaData = &'static mut TestMd;

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

pub struct TestDataBox {
    // nodes: Vec<TestMd>,
}

impl TestDataBox {
    // fn new() -> TestDataBox {
    //     TestDataBox {
    //         nodes: vec![TestMd::new(); 512],
    //     }
    // }
}

static mut MDS: [TestMd; 32] = [TestMd::new(); 32];
// static mut MDS: [TestMd; 32] =

impl IBuddyMdAdapter for TestDataBox {
    type Interface = TestMd;

    type OUT = &'static mut TestMd;

    fn get_md(n: u64) -> Option<Self::OUT> {
        unsafe { Some(&mut MDS[n as usize]) }
    }
    // fn mut_md<'a>(n: u64) -> Option<&'a mut Self::MetaData> {
    //     unsafe { Some(&mut MDS[n as usize]) }
    // }
}

#[test]
pub fn logic_test() {
    // let mut rf = TestDataBox::new();

    // let r = &mut rf.nodes[0];
    // r.last = 90;

    // let f = &mut rf.nodes[0];
    let mut bb: BuddyBase<TestDataBox, 4> = BuddyBase::new();

    bb.dump();
    println!("\n\n");

    bb.push(0);
    bb.push(1);
    bb.push(2);
    bb.push(3);
    bb.push(4);
    bb.push(5);
    bb.push(6);
    bb.push(7);
    bb.push(8);
    bb.push(9);
    bb.push(10);
    bb.push(11);
    bb.push(12);
    bb.push(13);
    bb.push(14);
    bb.push(15);

    bb.dump();
    println!("\n\n");
    let rfv = bb.pop(0).unwrap();
    let rfv2 = bb.pop(0).unwrap();
    let f = bb.pop(2).unwrap();
    let f1 = bb.pop(2).unwrap();

    bb.dump();

    println!("\n\n\n\n");

    bb.push(rfv);
    bb.push(rfv2);
    bb.push(f);
    bb.push(f1);

    bb.dump();

    println!("\n\n");

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
