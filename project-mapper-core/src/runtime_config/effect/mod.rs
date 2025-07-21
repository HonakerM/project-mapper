#[path = "./common.rs"]
pub mod common;

#[path = "./balance.rs"]
pub mod balance;

// reexport output component config to make it easier
pub use common::EffectComponentConfig;
