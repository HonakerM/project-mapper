use eframe::egui::{self, Response, Ui, Widget};
use project_mapper_core::config::{
    options::{RegionTypeOptions, SinkTypeOptions, SourceTypeOptions},
    runtime::RegionConfig,
    sink::{MonitorInfo, SinkConfig},
    source::SourceConfig,
};

use crate::config::{consts::WINDOWED_FULLSCREEN_MODE, parser::ParsedAvailableConfig};

use super::{region::DisplayElementWidget, sink::MonitorElementWidget, source::URIElementWidget};

#[derive(Clone)]
pub struct MonitorElementConfig {
    pub mode: String,
    pub monitor: MonitorInfo,
}

#[derive(Clone)]
pub enum SinkElementType {
    Monitor(MonitorElementConfig),
}

impl Default for MonitorElementConfig {
    fn default() -> Self {
        MonitorElementConfig {
            mode: WINDOWED_FULLSCREEN_MODE.to_owned(),
            monitor: MonitorInfo {
                name: "".to_owned(),
                resolution: "".to_owned(),
                refresh_rate_hz: 0,
            },
        }
    }
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
pub struct DisplayElementConfig {
    pub source: Option<UiElementInfo>,
    pub sink: Option<UiElementInfo>,
    pub element_infos: Option<Vec<UiElementInfo>>,
}

impl DisplayElementConfig {
    pub fn update_elements(&mut self, new_elements: Option<Vec<UiElementInfo>>) {
        self.element_infos = new_elements;
    }
}
impl Default for DisplayElementConfig {
    fn default() -> Self {
        DisplayElementConfig {
            source: None,
            sink: None,
            element_infos: None,
        }
    }
}

#[derive(Clone)]
pub enum RegionElementType {
    Display(DisplayElementConfig),
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
    pub region: RegionElementType,
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

#[derive(Clone, PartialEq, Eq)]
pub enum UiElementInfo {
    Source { id: u32, name: String },
    Sink { id: u32, name: String },
    Region { id: u32, name: String },
}

impl UiElementInfo {
    pub fn id(&self) -> u32 {
        match &self {
            UiElementInfo::Region { id, name } => id.clone(),
            UiElementInfo::Sink { id, name } => id.clone(),
            UiElementInfo::Source { id, name } => id.clone(),
        }
    }
    pub fn name(&self) -> String {
        match &self {
            UiElementInfo::Region { id, name } => name.clone(),
            UiElementInfo::Sink { id, name } => name.clone(),
            UiElementInfo::Source { id, name } => name.clone(),
        }
    }
}

pub struct UiElementData {
    pub data: ElementData,
}

impl UiElementData {
    pub fn name(&self) -> String {
        self.data.name()
    }

    pub fn id(&self) -> u32 {
        self.data.id()
    }

    pub fn info(&self) -> UiElementInfo {
        match &self.data {
            ElementData::Region(region_config) => UiElementInfo::Region {
                id: self.id(),
                name: self.name(),
            },
            ElementData::Sink(sink_config) => UiElementInfo::Sink {
                id: self.id(),
                name: self.name(),
            },
            ElementData::Source(source_config) => UiElementInfo::Source {
                id: self.id(),
                name: self.name(),
            },
        }
    }
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
        let id = self.data.id();
        let name = self.data.name();
        let mut ui_builder = egui::UiBuilder::new().id_salt(id);
        ui.scope_builder(ui_builder, |ui| {
            self.frame.show(ui, |ui| match &mut self.data.data {
                ElementData::Sink(sink_element) => match &mut sink_element.sink {
                    SinkElementType::Monitor(monitor_config) => {
                        ui.label(format!("Monitor Element {}", name));
                        let widget = MonitorElementWidget::new(self.config.clone(), sink_element)
                            .expect("uh oh");

                        ui.add(widget);
                    }
                },
                ElementData::Source(source_element) => match &mut source_element.source {
                    SourceElementType::URI(uri_config) => {
                        ui.label(format!("Uri Source {}", name));
                        let widget = URIElementWidget::new(self.config.clone(), source_element)
                            .expect("uh oh");

                        ui.add(widget);
                    }
                    _ => {}
                },
                ElementData::Region(region_element) => match &mut region_element.region {
                    RegionElementType::Display(display) => {
                        ui.label(format!("Display Region {}", name));
                        let widget = DisplayElementWidget::new(self.config.clone(), region_element)
                            .expect("uh oh");

                        ui.add(widget);
                    }
                    _ => {}
                },
            });
        })
        .response
    }
}
