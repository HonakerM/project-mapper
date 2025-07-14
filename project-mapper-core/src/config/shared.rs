pub type Uid = u32;

//Common trait for all types of components
pub trait Component {
    fn name(self) -> String;
    fn uid(self) -> Uid;
}
