use serde::{Deserialize, Serialize};

use super::{
    sink::{MonitorInfo, Resolution, SinkConfig, SinkType},
    source::{SourceConfig, SourceType},
};

#[derive(Serialize, Deserialize)]
pub struct DisplayRegion {
    pub source: u32,
    pub sink: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RegionType {
    Display(DisplayRegion),
}

#[derive(Serialize, Deserialize)]
pub struct RegionConfig {
    //region: ?,
    pub name: String,
    pub id: u32,
    pub region: RegionType,
}
