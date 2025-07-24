#[path = "./shared.rs"]
pub mod shared;

#[cfg(feature = "http-receiver")]
#[path = "./http.rs"]
pub mod http;
