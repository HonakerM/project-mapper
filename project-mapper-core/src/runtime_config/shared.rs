use std::{
    any::Any,
    fmt::Debug,
    hash::Hash,
    ops::{RangeFrom, RangeTo},
};

pub type Uid = i32;

pub static UID_MAX: Uid = std::i32::MAX;
pub static UID_MIN: Uid = 0;
pub static RESTRICTED_RANGE: RangeTo<Uid> = (..0);
pub static AVAILABLE_RANGE: RangeFrom<Uid> = (0..);

//Common trait for all types of components
pub trait ComponentConfig: Debug {
    fn name(&self) -> String;
    fn uid(&self) -> Uid;
    fn as_any(&self) -> &dyn Any;
    fn dependents(&self) -> Vec<Uid>;
    fn clone_box(&self) -> Box<dyn ComponentConfig>;
}

impl Clone for Box<dyn ComponentConfig> {
    fn clone(&self) -> Box<dyn ComponentConfig> {
        self.clone_box()
    }
}

impl PartialEq for Box<dyn ComponentConfig> {
    fn eq(&self, other: &Self) -> bool {
        self.uid() == other.uid()
    }
}
impl Eq for Box<dyn ComponentConfig> {}

impl Hash for Box<dyn ComponentConfig> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uid().hash(state)
    }
}
