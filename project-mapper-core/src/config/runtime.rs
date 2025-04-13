use serde::{Deserialize, Serialize};

use super::{
    sink::{MonitorInfo, Resolution, SinkConfig, SinkType},
    source::{SourceConfig, SourceType},
    region::{RegionConfig},
};


#[derive(Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub sinks: Vec<SinkConfig>,
    pub sources: Vec<SourceConfig>,
    pub regions: Vec<RegionConfig>,
}
