use std::sync::{Arc, Mutex};

use anyhow::Error;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Test {}

#[derive(Serialize, Deserialize)]
pub struct URI {
    pub uri: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SourceType {
    Test(Test),
    URI(URI),
}

#[derive(Serialize, Deserialize)]
pub struct SourceConfig {
    pub name: String,
    pub id: u32,
    pub source: SourceType,
}
