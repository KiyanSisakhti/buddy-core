use crate::IBuddyMetaData;

pub trait IBuddyMdAdapter {
    type MetaData: IBuddyMetaData;

    fn ref_md<'a>(n: u64) -> Option<&'a Self::MetaData>;
    fn mut_md<'a>(n: u64) -> Option<&'a mut Self::MetaData>;
}
