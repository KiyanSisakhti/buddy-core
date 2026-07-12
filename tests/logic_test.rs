use buddy_allocation_adapter::{BuddyBase, IBuddyMdAdapter, IBuddyMetaData, utils::buddy_lookup};

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
    fn get_next(&self) -> Option<u64> {
        self.next
    }

    fn set_next(&mut self, n: Option<u64>) {
        self.next = n;
    }

    fn get_last(&self) -> Option<u64> {
        self.last
    }

    fn set_last(&mut self, n: Option<u64>) {
        self.last = n;
    }

    fn set_order(&mut self, order: u8) {
        self.order = order;
    }

    fn get_order(&self) -> u8 {
        self.order
    }

    fn is_linked(&self) -> bool {
        self.is_linked
    }

    fn set_link(&mut self, state: bool) {
        self.is_linked = state;
    }

    fn get_ceil_reduction(&self) -> u8 {
        self.ceil_reduct
    }

    fn set_ceil_reduction(&mut self, ceil_reduct: u8) {
        self.ceil_reduct = ceil_reduct;
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

impl IBuddyMdAdapter for TestDataBox {
    type MetaData = TestMd;

    fn ref_md<'a>(n: u64) -> Option<&'a Self::MetaData> {
        unsafe { Some(&MDS[n as usize]) }
    }

    fn mut_md<'a>(n: u64) -> Option<&'a mut Self::MetaData> {
        unsafe { Some(&mut MDS[n as usize]) }
    }
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
