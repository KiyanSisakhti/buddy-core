pub trait IBuddyMetaData {
    fn get_next(&self) -> u64;
    fn set_next(&mut self, n: u64);

    fn get_last(&self) -> u64;
    fn set_last(&mut self, n: u64);

    fn set_order(&mut self, order: u8);
    fn get_order(&self) -> u8;

    fn is_linked(&self) -> bool;
    fn set_link(&mut self, state: bool);
}
