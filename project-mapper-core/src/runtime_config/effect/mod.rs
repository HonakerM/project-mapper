#[path = "./common.rs"]
pub mod common;

#[path = "./balance.rs"]
pub mod balance;

#[path = "./gamma.rs"]
pub mod gamma;

// reexport output component config to make it easier
pub use common::EffectComponentConfig;
