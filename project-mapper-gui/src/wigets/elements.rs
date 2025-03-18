use eframe::egui::{self, Response, Ui, Widget};
use project_mapper_core::config::{
    options::{RegionTypeOptions, SinkTypeOptions, SourceTypeOptions},
    runtime::RegionConfig,
    sink::{MonitorInfo, SinkConfig},
    source::SourceConfig,
};

use crate::config::parser::ParsedAvailableConfig;

use super::sink::MonitorElementWidget;

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

pub struct UiElementWidget<'a> {
    data: &'a mut UiElementData,
    pub config: ParsedAvailableConfig,
    pub frame: egui::Frame,
}

impl<'a> UiElementWidget<'a> {
    pub fn new(data: &'a mut UiElementData, config: ParsedAvailableConfig) -> Self {
        Self {
            data: data,
            config: config,
            frame: egui::Frame::new()
                .inner_margin(12)
                .outer_margin(24)
                .corner_radius(14)
                .shadow(egui::Shadow {
                    offset: [8, 12],
                    blur: 16,
                    spread: 0,
                    color: egui::Color32::from_black_alpha(180),
                })
                .fill(egui::Color32::from_rgba_unmultiplied(97, 0, 255, 128))
                .stroke(egui::Stroke::new(1.0, egui::Color32::GRAY)),
        }
    }
}
impl<'a> Widget for UiElementWidget<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut ui_builder = egui::UiBuilder::new().id_salt(self.data.data.id());
        ui.scope_builder(ui_builder, |ui| match &mut self.data.data {
            ElementData::Sink(sink_element) => match &mut sink_element.sink {
                SinkElementType::Monitor(monitor_config) => {
                    let widget = MonitorElementWidget::new(self.config.clone(), sink_element)
                        .expect("uh oh");

                    ui.add(widget);
                }
            },
            _ => {}
        })
        .response
    }
}
