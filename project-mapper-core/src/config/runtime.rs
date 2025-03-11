use serde::{Deserialize, Serialize};

use super::{
    sink::{MonitorInfo, Resolution, SinkConfig, SinkType},
    source::{SourceConfig, SourceType},
};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RegionType {
    Display { source: u32, sink: u32 },
}

#[derive(Serialize, Deserialize)]
pub struct RegionConfig {
    //region: ?,
    pub name: String,
    pub id: u32,
    pub region: RegionType,
}

#[derive(Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub sinks: Vec<SinkConfig>,
    pub sources: Vec<SourceConfig>,
    pub regions: Vec<RegionConfig>,
}
