use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Hash, PartialEq, Debug)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Eq for Resolution {}

#[derive(Serialize, Deserialize, Clone)]
pub struct MonitorInfo {
    pub name: String,
    pub resolution: Resolution,
    pub refresh_rate: u32,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum FullScreenMode {
    Windowed {},
    Borderless { name: String },
    Exclusive { info: MonitorInfo },
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum SinkType {
    OpenGLWindow { full_screen: FullScreenMode },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SinkConfig {
    pub name: String,
    pub id: u32,
    pub sink: SinkType,
}
