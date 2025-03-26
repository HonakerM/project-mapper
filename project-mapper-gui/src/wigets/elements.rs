use std::sync::mpsc::{Receiver, Sender};

use anyhow::{Error, Result};
use eframe::egui::{self, Response, Ui, Widget};
use project_mapper_core::config::{
    options::{RegionType, RegionTypeOptions, SinkTypeOptions, SourceTypeOptions},
    runtime::RegionConfig,
    sink::{FullScreenMode, MonitorInfo, SinkConfig, SinkType},
    source::{SourceConfig, SourceType},
};
use rand::distr::Alphanumeric;
use strum::IntoEnumIterator;
use strum_macros::Display;

use super::{region::DisplayElementWidget, sink::MonitorElementWidget, source::URIElementWidget};
use crate::{
    config::{
        consts::{BORDERLESS_FULLSCREEN_MODE, EXCLUSIVE_FULLSCREEN_MODE, WINDOWED_FULLSCREEN_MODE},
        parser::ParsedAvailableConfig,
    },
    core::simple_ui::UiEvent,
};
use rand::prelude::*;

#[derive(Clone, Debug)]
pub struct MonitorElementConfig {
    pub mode: String,
    pub monitor: MonitorInfo,
}

impl MonitorElementConfig {
    pub fn to_fullscreen_config(&self) -> Result<FullScreenMode> {
        if self.mode == WINDOWED_FULLSCREEN_MODE {
            Ok(FullScreenMode::Windowed {})
        } else if self.mode == BORDERLESS_FULLSCREEN_MODE {
            Ok(FullScreenMode::Borderless {
                name: self.monitor.name.clone(),
            })
        } else if self.mode == EXCLUSIVE_FULLSCREEN_MODE {
            Ok(FullScreenMode::Exclusive {
                info: self.monitor.clone(),
            })
        } else {
            Err(Error::msg(format!("Unknown mode {}", self.mode)))
        }
    }
    pub fn from_fullscreen_config(mode: &FullScreenMode) -> Self {
        match mode {
            FullScreenMode::Windowed {} => MonitorElementConfig {
                mode: WINDOWED_FULLSCREEN_MODE.to_owned(),
                monitor: MonitorInfo {
                    name: "".to_owned(),
                    resolution: "".to_owned(),
                    refresh_rate_hz: 0,
                },
            },
            FullScreenMode::Borderless { name } => MonitorElementConfig {
                mode: BORDERLESS_FULLSCREEN_MODE.to_owned(),
                monitor: MonitorInfo {
                    name: name.clone(),
                    resolution: "".to_owned(),
                    refresh_rate_hz: 0,
                },
            },
            FullScreenMode::Exclusive { info } => MonitorElementConfig {
                mode: EXCLUSIVE_FULLSCREEN_MODE.to_owned(),
                monitor: info.clone(),
            },
        }
    }
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

#[derive(Clone, Debug, strum_macros::Display, strum_macros::EnumIter)]
pub enum SinkElementType {
    Empty(),
    Monitor(MonitorElementConfig),
}

#[derive(Clone, Debug)]
pub struct TestElementConfig {}

impl Default for TestElementConfig {
    fn default() -> Self {
        Self {}
    }
}

#[derive(Clone, Debug)]
pub struct UriElementConfig {
    pub uri: String,
}

impl Default for UriElementConfig {
    fn default() -> Self {
        Self { uri: "".to_owned() }
    }
}

#[derive(Clone, strum_macros::Display, Debug, strum_macros::EnumIter)]
pub enum SourceElementType {
    Empty(),
    URI(UriElementConfig),
    Test(TestElementConfig),
}

#[derive(Clone, Debug)]
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
#[derive(Clone, Debug, strum_macros::Display, strum_macros::EnumIter)]
pub enum RegionElementType {
    Empty(),
    Display(DisplayElementConfig),
}

#[derive(strum_macros::Display, Debug)]
pub enum ElementData {
    Sink(SinkElementType),
    Source(SourceElementType),
    Region(RegionElementType),
}

impl ElementData {
    pub fn from_source_config(config: &SourceConfig) -> Self {
        let element_default: SourceElementType = match &config.source {
            SourceType::Test(config) => {
                let config = TestElementConfig {};
                SourceElementType::Test(TestElementConfig::from(config))
            }
            SourceType::URI(uri) => {
                let config = UriElementConfig {
                    uri: uri.uri.clone(),
                };
                SourceElementType::URI(config)
            }
        };
        Self::Source(element_default)
    }
    pub fn from_region_config(config: &RegionConfig) -> Self {
        let element_default: RegionElementType = match &config.region {
            project_mapper_core::config::runtime::RegionType::Display { source, sink } => {
                RegionElementType::Display(DisplayElementConfig {
                    source: Some(UiElementInfo::Source {
                        id: *source,
                        name: "".to_owned(),
                    }),
                    sink: Some(UiElementInfo::Sink {
                        id: *sink,
                        name: "".to_owned(),
                    }),
                    element_infos: None,
                })
            }
        };
        Self::Region(element_default)
    }
    pub fn from_sink_config(config: &SinkConfig) -> Self {
        let element_default: SinkElementType = match &config.sink {
            SinkType::OpenGLWindow { full_screen } => {
                let config = MonitorElementConfig::from_fullscreen_config(full_screen);
                SinkElementType::Monitor(config)
            }
        };
        Self::Sink(element_default)
    }

    pub fn element_type(&self) -> String {
        match self {
            ElementData::Region(config) => config.to_string(),
            ElementData::Sink(config) => config.to_string(),
            ElementData::Source(config) => config.to_string(),
        }
    }
    pub fn possible_element_types(&self) -> Vec<String> {
        match self {
            ElementData::Region(config) => {
                RegionElementType::iter().map(|x| x.to_string()).collect()
            }
            ElementData::Sink(config) => SinkElementType::iter().map(|x| x.to_string()).collect(),
            ElementData::Source(config) => {
                SourceElementType::iter().map(|x| x.to_string()).collect()
            }
        }
    }

    pub fn switch_element_type(&mut self, type_name: String) {
        // if we're already the right type do nothing
        if type_name == self.element_type() {
            return;
        }
        match self {
            ElementData::Region(config) => {
                if type_name == "Display" {
                    *self = ElementData::Region(RegionElementType::Display(
                        DisplayElementConfig::default(),
                    ));
                } else {
                    *self = ElementData::Region(RegionElementType::Empty());
                }
            }
            ElementData::Sink(config) => {
                if type_name == "Monitor" {
                    *self = ElementData::Sink(SinkElementType::Monitor(
                        MonitorElementConfig::default(),
                    ));
                } else {
                    *self = ElementData::Sink(SinkElementType::Empty());
                }
            }
            ElementData::Source(config) => {
                if type_name == "Test" {
                    *self =
                        ElementData::Source(SourceElementType::Test(TestElementConfig::default()));
                } else if type_name == "URI" {
                    *self =
                        ElementData::Source(SourceElementType::URI(UriElementConfig::default()));
                } else {
                    *self = ElementData::Source(SourceElementType::Empty());
                }
            }
        }
    }
}

#[derive(Clone, Eq, Debug)]
pub enum UiElementInfo {
    Source { id: u32, name: String },
    Sink { id: u32, name: String },
    Region { id: u32, name: String },
}

impl UiElementInfo {
    pub fn set_id(&mut self, new_id: u32) {
        match self {
            UiElementInfo::Region { id, name } => *id = new_id,
            UiElementInfo::Sink { id, name } => *id = new_id,
            UiElementInfo::Source { id, name } => *id = new_id,
        }
    }
    pub fn id(&self) -> u32 {
        match &self {
            UiElementInfo::Region { id, name } => id.clone(),
            UiElementInfo::Sink { id, name } => id.clone(),
            UiElementInfo::Source { id, name } => id.clone(),
        }
    }

    pub fn set_name(&mut self, new_name: String) {
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

#[derive(Debug)]
pub struct UiElementData {
    pub name: String,
    pub id: u32,
    pub data_type: String,
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
    pub event_sender: Sender<UiEvent>,
    pub config: ParsedAvailableConfig,
    pub frame: egui::Frame,
}

impl<'a> UiElementWidget<'a> {
    pub fn new(
        data: &'a mut UiElementData,
        event_sender: Sender<UiEvent>,
        config: ParsedAvailableConfig,
    ) -> Self {
        data.data.switch_element_type(data.data_type.clone());
        Self {
            data: data,
            event_sender: event_sender,
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
        let info = self.data.info();
        let id = self.data.id();
        let mut ui_builder = egui::UiBuilder::new().id_salt(id);
        ui.scope_builder(ui_builder, |ui| {
            self.frame.show(ui, |ui| {
                egui::Grid::new("element_grid")
                    .num_columns(2)
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Name");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.data.name).desired_width(120.0),
                        );
                        let mut button = ui.button("x");
                        if button.clicked() {
                            self.event_sender
                                .send(UiEvent::DeleteElement(info.clone()))
                                .unwrap();
                        }

                        ui.end_row();

                        ui.label("Type");
                        let current_type = self.data.data.element_type();
                        egui::ComboBox::from_id_salt("Type")
                            .selected_text(format!("{current_type}"))
                            .show_ui(ui, |ui| {
                                for element_modes in self.data.data.possible_element_types() {
                                    ui.selectable_value(
                                        &mut self.data.data_type,
                                        element_modes.clone(),
                                        element_modes.clone(),
                                    );
                                }
                            });
                    });

                match &mut self.data.data {
                    ElementData::Sink(sink_element) => match sink_element {
                        SinkElementType::Monitor(monitor_config) => {
                            let widget = MonitorElementWidget::new(self.config.clone(), self.data)
                                .expect("uh oh");

                            ui.add(widget);
                        }
                        _ => {}
                    },
                    ElementData::Source(source_element) => match source_element {
                        SourceElementType::URI(uri_config) => {
                            let widget = URIElementWidget::new(self.config.clone(), self.data)
                                .expect("uh oh");

                            ui.add(widget);
                        }
                        _ => {}
                    },
                    ElementData::Region(region_element) => match region_element {
                        RegionElementType::Display(display) => {
                            let widget = DisplayElementWidget::new(self.config.clone(), self.data)
                                .expect("uh oh");

                            ui.add(widget);
                        }
                        _ => {}
                    },
                }
            });
        })
        .response
    }
}

pub struct AddElementWidget {
    event_sender: Sender<UiEvent>,
    default: UiElementInfo,
    pub config: ParsedAvailableConfig,
}

impl AddElementWidget {
    pub fn new(
        event_sender: Sender<UiEvent>,
        config: ParsedAvailableConfig,
        default: UiElementInfo,
    ) -> Self {
        Self {
            event_sender: event_sender,
            config: config,
            default: default,
        }
    }
}
impl Widget for AddElementWidget {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical_centered(|ui| {
            //let mut button = ui.button(egui_material_icons::icons::ICON_ADD);
            let mut button = ui.button("X");
            if button.clicked() {
                let mut rng = rand::rng();
                let id: u32 = rng.random();
                let name: String = rng
                    .sample_iter(&Alphanumeric)
                    .take(5)
                    .map(char::from)
                    .collect();

                let mut current_info = self.default.clone();
                current_info.set_id(id);
                current_info.set_name(name);

                self.event_sender
                    .send(UiEvent::NewElement(current_info))
                    .unwrap();
            }
        })
        .response
    }
}
