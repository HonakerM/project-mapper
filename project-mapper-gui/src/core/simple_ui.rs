use std::sync::mpsc::{Receiver, Sender};

use eframe::{
    self, App,
    egui::{self, Response, TextBuffer, Widget},
};
use project_mapper_core::config::{
    runtime::{RegionConfig, RegionType, RuntimeConfig},
    sink::{MonitorInfo, Resolution, SinkConfig, SinkType},
    source::{SourceConfig, SourceType, Test, URI},
};
use rand::distr::slice::Empty;

use crate::{
    config::{
        consts::{BORDERLESS_FULLSCREEN_MODE, EXCLUSIVE_FULLSCREEN_MODE, WINDOWED_FULLSCREEN_MODE},
        parser::ParsedAvailableConfig,
    },
    runtime_api,
    wigets::{
        elements::{
            AddElementWidget, DisplayElementConfig, ElementData, MonitorElementConfig,
            RegionElementType, SinkElementType, SourceElementType, UiElementData, UiElementInfo,
            UiElementWidget, UriElementConfig,
        },
        sink::MonitorElementWidget,
    },
};
use anyhow::{Error, Result};

use super::app::{CoreApp, CoreView};

pub enum UiEvent {
    NewElement(UiElementInfo),
    DeleteElement(UiElementInfo),
}

pub struct SimpleUiCore {
    config: ParsedAvailableConfig,
    event_receiver: Receiver<UiEvent>,
    event_sender: Sender<UiEvent>,
    elements: Vec<UiElementData>,
    element_infos: Vec<UiElementInfo>,
}

impl SimpleUiCore {
    pub fn new(config: ParsedAvailableConfig) -> Result<SimpleUiCore> {
        let source_data = UiElementData {
            name: "source".to_owned(),
            id: 1,
            data: ElementData::Source(SourceElementType::URI(UriElementConfig::default())),
            data_type: SourceElementType::URI(UriElementConfig::default()).to_string(),
        };

        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        let elm = UiElementData {
            name: "sink1".to_owned(),
            id: 2,
            data: ElementData::Sink(SinkElementType::Monitor(MonitorElementConfig::default())),
            data_type: SinkElementType::Monitor(MonitorElementConfig::default()).to_string(),
        };
        let elm_2 = UiElementData {
            name: "sink2".to_owned(),
            id: 3,
            data: ElementData::Sink(SinkElementType::Monitor(MonitorElementConfig::default())),
            data_type: SinkElementType::Monitor(MonitorElementConfig::default()).to_string(),
        };
        let elm_3 = UiElementData {
            name: "region_1".to_owned(),
            id: 4,
            data: ElementData::Region(RegionElementType::Display(DisplayElementConfig::default())),
            data_type: RegionElementType::Display(DisplayElementConfig::default()).to_string(),
        };

        let (tx, rx) = std::sync::mpsc::channel();

        Ok(Self {
            config: config,
            event_sender: tx,
            event_receiver: rx,
            elements: vec![source_data, elm, elm_2, elm_3],
            element_infos: vec![],
        })
    }

    pub fn refresh_infos(&mut self) {
        let mut element_infos = vec![];
        for element in &mut self.elements {
            element_infos.push(element.info());
        }
        self.element_infos = element_infos;
    }

    pub fn refresh_events(&mut self) {
        loop {
            match self.event_receiver.try_recv() {
                Ok(msg) => {
                    // add to buffer/queue but process all when buffer.len() = 100
                    match msg {
                        UiEvent::NewElement(element_info) => match element_info {
                            UiElementInfo::Source { id, name } => {
                                self.elements.push(UiElementData {
                                    id: id,
                                    name: name,
                                    data: ElementData::Source(SourceElementType::Empty()),
                                    data_type: SourceElementType::Empty().to_string(),
                                });
                            }
                            UiElementInfo::Region { id, name } => {
                                self.elements.push(UiElementData {
                                    id: id,
                                    name: name,

                                    data: ElementData::Region(RegionElementType::Empty()),
                                    data_type: RegionElementType::Empty().to_string(),
                                });
                            }
                            UiElementInfo::Sink { id, name } => {
                                self.elements.push(UiElementData {
                                    id: id,
                                    name: name,
                                    data: ElementData::Sink(SinkElementType::Empty()),
                                    data_type: SinkElementType::Empty().to_string(),
                                });
                            }
                        },
                        UiEvent::DeleteElement(element) => {
                            if let Some(del_id) = self
                                .elements
                                .iter()
                                .position(|item| item.id() == element.id())
                            {
                                self.elements.remove(del_id);
                            }
                        }
                    }
                }
                Err(e) => {
                    break;
                }
            }
        }
    }

    pub fn refresh(&mut self) {
        self.refresh_events();
        self.refresh_infos();
    }
}

impl CoreView for &mut SimpleUiCore {
    fn config(self) -> Result<RuntimeConfig> {
        let mut sinks: Vec<SinkConfig> = Vec::new();
        let mut sources: Vec<SourceConfig> = Vec::new();
        let mut regions: Vec<RegionConfig> = Vec::new();

        let mut parse_error = None;
        for element in &self.elements {
            let name = element.name().clone();
            let id = element.id();
            match &element.data {
                ElementData::Sink(sink_config) => match sink_config {
                    SinkElementType::Monitor(monitor_config) => {
                        sinks.push(SinkConfig {
                            name: name,
                            id: id,
                            sink: SinkType::OpenGLWindow {
                                full_screen: monitor_config.to_fullscreen_config()?,
                            },
                        });
                    }
                    _ => {}
                },
                ElementData::Source(source_config) => match source_config {
                    SourceElementType::URI(config) => {
                        sources.push(SourceConfig {
                            name: name,
                            id: id,
                            source: SourceType::URI(URI {
                                uri: config.uri.clone(),
                            }),
                        });
                    }
                    SourceElementType::Test(config) => {
                        sources.push(SourceConfig {
                            name: name,
                            id: id,
                            source: SourceType::Test(Test {}),
                        });
                    }
                    _ => {}
                },
                ElementData::Region(region_config) => match region_config {
                    RegionElementType::Display(display) => {
                        let mut src = &UiElementInfo::Source {
                            id: 0,
                            name: "".to_owned(),
                        };
                        if let Some(element) = &display.source {
                            src = element;
                        } else {
                            parse_error = Some(Error::msg("Must have source selected"));
                            break;
                        }

                        let mut sink = &UiElementInfo::Sink {
                            id: 0,
                            name: "".to_owned(),
                        };
                        if let Some(element) = &display.sink {
                            sink = element;
                        } else {
                            parse_error = Some(Error::msg("Must have sink selected"));
                            break;
                        }
                        regions.push(RegionConfig {
                            name: name,
                            id: id,
                            region: RegionType::Display {
                                source: src.id(),
                                sink: sink.id(),
                            },
                        });
                    }
                    _ => {}
                },
            }
        }

        if let Some(error) = parse_error {
            Err(error)
        } else {
            Ok(RuntimeConfig {
                sinks: sinks,
                sources: sources,
                regions: regions,
            })
        }
    }
    fn load_config(self, config: RuntimeConfig) -> Result<()> {
        let mut new_elements: Vec<UiElementData> = vec![];
        for source in config.sources {
            let data = ElementData::from_source_config(&source);
            let ui_data = UiElementData {
                name: source.name,
                id: source.id,
                data_type: data.element_type(),
                data: data,
            };
            new_elements.push(ui_data);
        }
        for sink in config.sinks {
            let data = ElementData::from_sink_config(&sink);
            let ui_data = UiElementData {
                name: sink.name,
                id: sink.id,
                data_type: data.element_type(),
                data: data,
            };
            new_elements.push(ui_data);
        }
        for region in config.regions {
            let data = ElementData::from_region_config(&region);
            let ui_data = UiElementData {
                name: region.name,
                id: region.id,
                data_type: data.element_type(),
                data: data,
            };
            new_elements.push(ui_data);
        }

        self.elements.clear();
        self.elements.append(&mut new_elements);
        Ok(())
    }
}

pub struct SimpleUiApp<'a> {
    pub core: &'a mut SimpleUiCore,
}

impl<'a> SimpleUiApp<'a> {
    pub fn new(core: &mut SimpleUiCore) -> SimpleUiApp {
        SimpleUiApp { core: core }
    }
}
impl<'a> Widget for SimpleUiApp<'a> {
    fn ui(mut self, ui: &mut egui::Ui) -> Response {
        self.core.refresh();

        let mut sink_elements: Vec<&mut UiElementData> = Vec::new();
        let mut source_elements: Vec<&mut UiElementData> = Vec::new();
        let mut region_elements: Vec<&mut UiElementData> = Vec::new();

        for element in &mut self.core.elements {
            match &mut element.data {
                ElementData::Sink(sink_config) => {
                    sink_elements.push(element);
                }
                ElementData::Source(source_config) => {
                    source_elements.push(element);
                }
                ElementData::Region(region_config) => {
                    match region_config {
                        RegionElementType::Display(display) => {
                            display.element_infos = Some(self.core.element_infos.clone());
                        }
                        _ => {}
                    }
                    region_elements.push(element);
                }
            }
        }

        let mut ui_builder = egui::UiBuilder::new();
        ui.scope_builder(ui_builder, |ui| {
            ui.heading("Simple UI For Project Mapper");

            ui.columns(3, |uis| {
                let (first_inst, remaining) = uis.split_at_mut(1);
                let (second_instance, third_instance) = remaining.split_at_mut(1);
                let source_ui = &mut first_inst[0];
                let region_ui = &mut second_instance[0];
                let sink_ui = &mut third_instance[0];

                for source_element in source_elements {
                    let mut widget = UiElementWidget::new(
                        source_element,
                        self.core.event_sender.clone(),
                        self.core.config.clone(),
                    );
                    source_ui.add(widget);
                }
                let mut src_add_button_widget = AddElementWidget::new(
                    self.core.event_sender.clone(),
                    self.core.config.clone(),
                    UiElementInfo::Source {
                        id: 0,
                        name: "".to_string(),
                    },
                );
                source_ui.add(src_add_button_widget);

                for region_element in region_elements {
                    let mut widget = UiElementWidget::new(
                        region_element,
                        self.core.event_sender.clone(),
                        self.core.config.clone(),
                    );
                    region_ui.add(widget);
                }
                let mut region_add_button_widget = AddElementWidget::new(
                    self.core.event_sender.clone(),
                    self.core.config.clone(),
                    UiElementInfo::Region {
                        id: 0,
                        name: "".to_string(),
                    },
                );
                region_ui.add(region_add_button_widget);

                for sink_element in sink_elements {
                    let mut widget = UiElementWidget::new(
                        sink_element,
                        self.core.event_sender.clone(),
                        self.core.config.clone(),
                    );
                    sink_ui.add(widget);
                }
                let mut sink_add_button_widget = AddElementWidget::new(
                    self.core.event_sender.clone(),
                    self.core.config.clone(),
                    UiElementInfo::Sink {
                        id: 0,
                        name: "".to_string(),
                    },
                );
                sink_ui.add(sink_add_button_widget);
            })
        })
        .response
    }
}
