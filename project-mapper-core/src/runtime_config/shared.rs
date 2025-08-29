use std::{any::Any, fmt::Debug};

pub type Uid = i32;

//Common trait for all types of components
pub trait ComponentConfig: Debug {
    fn name(&self) -> String;
    fn uid(&self) -> Uid;
    fn as_any(&self) -> &dyn Any;
    fn dependents(&self) -> Vec<Uid>;
}
