use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct MonitorInfo {
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum SinkType {
    OpenGLWindow { monitor: Option<MonitorInfo> },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SinkConfig {
    pub name: String,
    pub id: u32,
    pub sink: SinkType,
}
