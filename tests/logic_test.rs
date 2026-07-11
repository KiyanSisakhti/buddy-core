use buddy_allocation_adapter::{BuddyBase, IBuddyMdAdapter, IBuddyMetaData};

#[derive(Debug, Clone, Copy)]
pub struct TestMd {
    next: u64,
    last: u64,

    is_linked: bool,

    order: u8,
}
impl TestMd {
    pub const fn new() -> Self {
        Self {
            next: u64::MAX,
            last: u64::MAX,
            is_linked: false,
            order: 0,
        }
    }
}

impl IBuddyMetaData for TestMd {
    fn get_next(&self) -> u64 {
        self.next
    }

    fn set_next(&mut self, n: u64) {
        self.next = n;
    }

    fn get_last(&self) -> u64 {
        self.last
    }

    fn set_last(&mut self, n: u64) {
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
}

pub struct TestDataBox {
    nodes: Vec<TestMd>,
}

impl TestDataBox {
    fn new() -> TestDataBox {
        TestDataBox {
            nodes: vec![TestMd::new(); 512],
        }
    }
}

static mut MDS: [TestMd; 512] = [TestMd::new(); 512];

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
    let mut bb = BuddyBase::<TestDataBox>::new();

    bb.put(0);
    bb.put(1);
    bb.put(2);

    // bb.dump(0);

    // println!("----- lo c ------{f:?}");
    println!("----- lo c ------");
    println!("----- lo c ------");
    println!("----- lo c ------");
    println!("----- lo c ------");
    println!("----- lo c ------");
    println!("----- lo c ------");
}
