pub trait IBuddyMetaData {
    type MetaData;

    fn get_next(md: &Self::MetaData) -> Option<u64>;
    fn set_next(md: &mut Self::MetaData, n: Option<u64>);

    fn get_last(md: &Self::MetaData) -> Option<u64>;
    fn set_last(md: &mut Self::MetaData, n: Option<u64>);

    fn set_order(md: &mut Self::MetaData, order: u8);
    fn get_order(md: &Self::MetaData) -> u8;

    fn get_ceil_reduction(md: &Self::MetaData) -> u8;
    fn set_ceil_reduction(md: &mut Self::MetaData, ceil_reduct: u8);

    fn is_linked(md: &Self::MetaData) -> bool;
    fn set_link(md: &mut Self::MetaData, state: bool);
}
