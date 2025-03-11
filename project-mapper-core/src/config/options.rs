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
pub struct WindowOptions {}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BorderlessOptions {
    pub monitors: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExclusiveOptions {
    pub monitor_configs: HashMap<String, HashMap<ResolutionJson, Vec<RefreshRate>>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum FullscreenOptions {
    Windowed(WindowOptions),
    Borderless(BorderlessOptions),
    Exclusive(ExclusiveOptions),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum SinkTypeOptions {
    OpenGLWindow {
        full_screen_modes: Vec<FullscreenOptions>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum SourceTypeOptions {
    Test {},
    URI { uri_types: Vec<String> },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum RegionTypeOptions {
    Display {},
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AvailableConfig {
    pub sinks: Vec<SinkTypeOptions>,
    pub sources: Vec<SourceTypeOptions>,
    pub regions: Vec<RegionTypeOptions>,
}
