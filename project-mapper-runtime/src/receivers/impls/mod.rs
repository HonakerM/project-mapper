#[path = "./shared.rs"]
pub mod shared;

#[cfg(feature = "http-receiver")]
#[path = "./http/mod.rs"]
pub mod http;
