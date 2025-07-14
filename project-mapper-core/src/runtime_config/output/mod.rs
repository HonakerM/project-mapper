#[path = "./common.rs"]
pub mod common;
#[path = "./window.rs"]
pub mod window;

// reexport output component config to make it easier
pub use common::OutputComponentConfig;
