#[path = "./handler.rs"]
pub mod handler;

#[path = "./config.rs"]
pub mod config;

#[path = "./utils.rs"]
pub mod utils;

pub(crate) use handler::Message;
pub use handler::WindowHandler;
