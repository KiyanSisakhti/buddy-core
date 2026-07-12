use crate::IBuddyMetaData;

pub trait IBuddyMdAdapter {
    type Interface: IBuddyMetaData<MetaData = Self::OUT>;
    type OUT;

    fn get_md(n: u64) -> Option<Self::OUT>;
}
