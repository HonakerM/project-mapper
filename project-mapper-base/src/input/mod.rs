#[path = "./test.rs"]
pub mod test;
pub use test::TestComponent;
pub use test::TestConfig;

#[path = "./uri.rs"]
pub mod uri;
pub use uri::UriComponent;
pub use uri::UriConfig;
