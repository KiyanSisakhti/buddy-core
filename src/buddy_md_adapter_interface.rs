use crate::IBuddyMetaData;

pub trait IBuddyMdAdapter {
    type Interface: IBuddyMetaData<MetaData = Self::MetaDataHandle>;
    type MetaDataHandle;

    fn get_md(n: u64) -> Option<Self::MetaDataHandle>;
}
