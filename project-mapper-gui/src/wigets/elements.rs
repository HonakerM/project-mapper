use eframe::egui::{self, Response, Ui, Widget};
use project_mapper_core::config::{
    options::{RegionTypeOptions, SinkTypeOptions, SourceTypeOptions},
    runtime::RegionConfig,
    sink::{MonitorInfo, SinkConfig},
    source::SourceConfig,
};

use crate::config::parser::ParsedAvailableConfig;

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

#[derive(Clone)]
pub struct SinkElementConfig {
    pub sink: SinkElementType,
}

#[derive(Clone)]
pub enum SourceElementType {
    URI(String),
    Test(),
}
#[derive(Clone)]
pub enum RegionElementType {
    Display {
        source: Option<UiElementInfo>,
        sink: Option<UiElementInfo>,
        element_infos: Option<Vec<UiElementInfo>>,
    },
}

#[derive(strum_macros::Display)]
pub enum ElementData {
    Sink(SinkElementType),
    Source(SourceElementType),
    Region(RegionElementType),
}

impl ElementData {
    pub fn element_type(&self) -> String {
        self.to_string()
    }
}

#[derive(Clone, Eq)]
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

    pub fn set_name(&mut self, new_name: String){
        match self {
            UiElementInfo::Region { id, name } => name.replace_range(.., new_name.as_str()),
            UiElementInfo::Sink { id, name } => name.replace_range(.., new_name.as_str()),
            UiElementInfo::Source { id, name } => name.replace_range(.., new_name.as_str()),
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

impl PartialEq for UiElementInfo {
    fn eq(&self, other: &UiElementInfo) -> bool {
        self.id() == other.id()
    }
}

pub struct UiElementData {
    pub name: String,
    pub id: u32,
    pub data: ElementData,
}

impl UiElementData {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn id(&self) -> u32 {
        self.id
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
        let mut ui_builder = egui::UiBuilder::new().id_salt(id);
        ui.scope_builder(ui_builder, |ui| {
            self.frame.show(ui, |ui| {

            egui::Grid::new(id)
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                ui.label(format!("{} Element", self.data.data.element_type()));
                ui.add(egui::TextEdit::singleline(&mut self.data.name));
            });

            match &mut self.data.data {
                ElementData::Sink(sink_element) => match sink_element {
                    SinkElementType::Monitor(monitor_config) => {
                        let widget = MonitorElementWidget::new(self.config.clone(), self.data)
                            .expect("uh oh");

                        ui.add(widget);
                    }
                },
                ElementData::Source(source_element) => match source_element {
                    SourceElementType::URI(uri_config) => {
                        let widget =
                            URIElementWidget::new(self.config.clone(), self.data).expect("uh oh");

                        ui.add(widget);
                    }
                    _ => {}
                },
                ElementData::Region(region_element) => match region_element {
                    RegionElementType::Display {
                        source,
                        sink,
                        element_infos,
                    } => {
                        let widget = DisplayElementWidget::new(self.config.clone(), self.data)
                            .expect("uh oh");

                        ui.add(widget);
                    }
                    _ => {}
                },
            }});
        })
        .response
    }
}
