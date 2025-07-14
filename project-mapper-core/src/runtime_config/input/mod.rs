#[path = "./common.rs"]
pub mod common;
#[path = "./test.rs"]
pub mod test;
#[path = "./uri.rs"]
pub mod uri;

// reexport output component config to make it easier
pub use common::InputComponentConfig;
