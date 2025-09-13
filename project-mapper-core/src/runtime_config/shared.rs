use std::{any::Any, fmt::Debug};

pub type Uid = i32;

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
