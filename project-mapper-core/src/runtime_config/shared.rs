use std::any::Any;

pub type Uid = u32;

//Common trait for all types of components
pub trait ComponentConfig {
    fn name(&self) -> String;
    fn uid(&self) -> Uid;
    fn as_any(&self) -> &dyn Any;
}
