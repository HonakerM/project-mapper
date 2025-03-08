use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum SourceType {
    Test {},
}

#[derive(Serialize, Deserialize)]
struct SourceConfig {
    name: String,
    id: u32,
    source: SourceType,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum SinkType {
    Monitor { name: String },
}

#[derive(Serialize, Deserialize)]
struct SinkConfig {
    name: String,
    id: u32,
    sink: SinkType,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum RegionType {
    Display { source: u32, sink: u32 },
}

#[derive(Serialize, Deserialize)]
struct RegionConfig {
    //region: ?,
    name: String,
    id: u32,
    region: RegionType,
}

/*
#[derive(Serialize, Deserialize)]
struct RuntimeConfig {
    sinks: [SinkConfig],
    sources: [SourceConfig],
    regions: [RegionConfig]
} */
