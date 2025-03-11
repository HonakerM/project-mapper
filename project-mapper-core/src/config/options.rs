use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{
    sink::{MonitorInfo, RefreshRate, Resolution, ResolutionJson, SinkConfig, SinkType},
    source::{SourceConfig, SourceType},
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum RegionType {
    Display { source: u32, sink: u32 },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WindowOptions {
    pub type_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BorderlessOptions {
    pub type_name: String,
    pub monitors: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExclusiveOptions {
    pub type_name: String,
    pub monitor_configs: HashMap<String, HashMap<ResolutionJson, Vec<RefreshRate>>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FullscreenOptions {
    pub windowed: WindowOptions,
    pub borderless: BorderlessOptions,
    pub exclusive: ExclusiveOptions,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum SinkTypeOptions {
    OpenGLWindow { full_screen: FullscreenOptions },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SinkOption {
    pub type_name: String,
    pub type_options: SinkTypeOptions,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum SourceTypeOptions {
    Test {},
    URI { uri_types: Vec<String> },
}
#[derive(Serialize, Deserialize, Debug)]
pub struct SourceOption {
    pub type_name: String,
    pub type_options: SourceTypeOptions,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum RegionTypeOptions {
    Display {},
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegionOption {
    pub type_name: String,
    pub type_options: RegionTypeOptions,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AvailableConfig {
    pub sinks: Vec<SinkOption>,
    pub sources: Vec<SourceOption>,
    pub regions: Vec<RegionOption>,
}
