use std::io::Sink;

use serde::{Deserialize, Serialize};
use serde_json::Result;

use super::{
    sink::{SinkConfig, SinkType},
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

impl RuntimeConfig {
    pub(crate) fn default() -> RuntimeConfig {
        let sourceOne: SourceConfig = SourceConfig {
            name: String::from("test source"),
            id: 1,
            source: SourceType::Test {},
        };
        let sources = Vec::from([sourceOne]);

        let sinkOne: SinkConfig = SinkConfig {
            name: String::from("main monitor"),
            id: 1,
            sink: SinkType::OpenGLWindow { monitor: None },
        };
        let sinkTwo: SinkConfig = SinkConfig {
            name: String::from("other monitor"),
            id: 2,
            sink: SinkType::OpenGLWindow { monitor: None },
        };
        let sinks = Vec::from([sinkOne]);

        let regionOne: RegionConfig = RegionConfig {
            name: String::from("main region"),
            id: 1,
            region: RegionType::Display { source: 1, sink: 1 },
        };
        let regions = Vec::from([regionOne]);

        let config = RuntimeConfig {
            sinks: sinks,
            sources: sources,
            regions: regions,
        };
        config
    }
}
