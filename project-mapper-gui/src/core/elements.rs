use project_mapper_core::config::{
    options::{RegionTypeOptions, SinkTypeOptions, SourceTypeOptions},
    runtime::RegionConfig,
    sink::SinkConfig,
    source::SourceConfig,
};

pub enum ElementData {
    Sink(SinkConfig),
    Source(SourceConfig),
    Region(RegionConfig),
}

pub struct UiElementData {
    pub element_type: String,
    pub data: ElementData,
}
