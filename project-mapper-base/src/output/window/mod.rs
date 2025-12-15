#[path = "./app.rs"]
pub mod app;

#[path = "./state.rs"]
pub mod state;

#[path = "./component.rs"]
pub mod component;

#[path = "./config.rs"]
pub mod config;

pub use component::WindowComponent;
