use project_mapper_core::config::{
    options::{RegionTypeOptions, SinkTypeOptions, SourceTypeOptions},
    runtime::RegionConfig,
    sink::{MonitorInfo, SinkConfig},
    source::SourceConfig,
};

#[derive(Clone)]
pub struct MonitorElementConfig {
    pub mode: String,
    pub monitor: MonitorInfo,
}

#[derive(Clone)]
pub enum SinkElementType {
    Monitor(MonitorElementConfig),
}

#[derive(Clone)]
pub struct SinkElementConfig {
    pub name: String,
    pub id: u32,
    pub sink: SinkElementType,
}

#[derive(Clone)]
pub enum SourceElementType {
    URI(String),
    Test(),
}
#[derive(Clone)]
pub enum RegionElementType {
    Display { source: u32, sink: u32 },
}

#[derive(Clone)]
pub struct SourceElementConfig {
    pub name: String,
    pub id: u32,
    pub source: SourceElementType,
}

#[derive(Clone)]
pub struct RegionElementConfig {
    pub name: String,
    pub id: u32,
    pub source: RegionElementType,
}

#[derive(strum_macros::Display)]
pub enum ElementData {
    Sink(SinkElementConfig),
    Source(SourceElementConfig),
    Region(RegionElementConfig),
}

impl ElementData {
    pub fn element_type(&self) -> String {
        self.to_string()
    }

    pub fn name(&self) -> String {
        match self {
            ElementData::Sink(config) => config.name.clone(),
            ElementData::Source(config) => config.name.clone(),
            ElementData::Region(config) => config.name.clone(),
        }
    }

    pub fn id(&self) -> u32 {
        match self {
            ElementData::Sink(config) => config.id.clone(),
            ElementData::Source(config) => config.id.clone(),
            ElementData::Region(config) => config.id.clone(),
        }
    }
}

pub struct UiElementData {
    pub data: ElementData,
}
