use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{
    sink::{MonitorInfo, RefreshRate, Resolution, SinkConfig, SinkType},
    source::{SourceConfig, SourceType},
};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RegionType {
    Display { source: u32, sink: u32 },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WindowOptions {
    pub type_name: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BorderlessOptions {
    pub type_name: String,
    pub monitors: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ExclusiveOptions {
    pub type_name: String,
    pub monitor_configs: HashMap<String, HashMap<Resolution, Vec<RefreshRate>>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FullscreenOptions {
    windowed: WindowOptions,
    borderless: BorderlessOptions,
    exclusive: ExclusiveOptions,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SinkTypeOptions {
    OpenGLWindow { full_screen: FullscreenOptions },
}

#[derive(Serialize, Deserialize)]
pub struct SinkOption {
    pub type_name: String,
    pub type_options: SinkTypeOptions,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum SourceTypeOptions {
    Test {},
    URI { uri_types: Vec<String> },
}
#[derive(Serialize, Deserialize)]
pub struct SourceOption {
    pub type_name: String,
    pub type_options: SourceTypeOptions,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum RegionTypeOptions {
    Display {},
}

#[derive(Serialize, Deserialize)]
pub struct RegionOptions {
    pub type_name: String,
    pub type_options: RegionTypeOptions,
}

#[derive(Serialize, Deserialize)]
pub struct AvailableConfig {
    pub sinks: Vec<SinkOption>,
    pub sources: Vec<SourceOption>,
    pub regions: Vec<RegionOptions>,
}
