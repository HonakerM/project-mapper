use std::io::Sink;

use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SinkType {
    OpenGLMonitor { name: String },
}

#[derive(Serialize, Deserialize)]
pub struct SinkConfig {
    pub name: String,
    pub id: u32,
    pub sink: SinkType,
}
