#[path = "./app.rs"]
pub mod app;

#[path = "./state.rs"]
pub mod state;

#[path = "./component.rs"]
pub mod component;

#[path = "./config.rs"]
pub mod config;

#[path = "./utils.rs"]
pub mod utils;

pub use component::WindowComponent;
