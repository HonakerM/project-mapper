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

impl RuntimeConfig {
    pub(crate) fn default() -> RuntimeConfig {
        let source_one: SourceConfig = SourceConfig {
            name: String::from("test source"),
            id: 1,
            source: crate::config::source::SourceType::Test(super::source::Test {}),
        };
        let source_two: SourceConfig = SourceConfig {
            name: String::from("uri source"),
            id: 2,
            source: crate::config::source::SourceType::URI(super::source::URI {
                uri: String::from(
                    "https://gstreamer.freedesktop.org/data/media/sintel_trailer-480p.webm",
                ),
            }),
        };
        let sources = Vec::from([source_one, source_two]);

        let sink_one: SinkConfig = SinkConfig {
            name: String::from("main monitor"),
            id: 1,
            sink: SinkType::OpenGLWindow {
                full_screen: super::sink::FullScreenMode::Windowed {},
            },
        };
        let sink_two: SinkConfig = SinkConfig {
            name: String::from("other monitor"),
            id: 2,
            sink: SinkType::OpenGLWindow {
                full_screen: super::sink::FullScreenMode::Borderless {
                    name: String::from("\\\\.\\DISPLAY1"),
                },
            },
        };
        // let sink_three: SinkConfig = SinkConfig {
        // name: String::from("other monitor"),
        // id: 3,
        // sink: SinkType::OpenGLWindow {
        // full_screen: super::sink::FullScreenMode::Exclusive {
        // info: MonitorInfo {
        // name: String::from("\\\\.\\DISPLAY2"),
        // resolution: Resolution {
        // width: 2560,
        // height: 1440,
        // },
        // refresh_rate: 120000,
        // },
        // },
        // },
        // };
        let sinks = Vec::from([sink_one, sink_two]);

        let regio_one: RegionConfig = RegionConfig {
            name: String::from("main region"),
            id: 1,
            region: RegionType::Display { source: 1, sink: 1 },
        };
        let region_two: RegionConfig = RegionConfig {
            name: String::from("secondary region"),
            id: 2,
            region: RegionType::Display { source: 2, sink: 2 },
        };
        let regions = Vec::from([regio_one, region_two]);

        let config = RuntimeConfig {
            sinks: sinks,
            sources: sources,
            regions: regions,
        };
        config
    }
}
