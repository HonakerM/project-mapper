use serde::{Deserialize, Serialize};

use super::{
    region::RegionConfig,
    sink::{MonitorInfo, Resolution, SinkConfig, SinkType},
    source::{SourceConfig, SourceType},
};

#[derive(Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub sinks: Vec<SinkConfig>,
    pub sources: Vec<SourceConfig>,
    pub regions: Vec<RegionConfig>,
}
