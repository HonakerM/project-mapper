use std::sync::{Arc, Mutex};

use eframe::{
    self, App,
    egui::{self, Response, TextBuffer, Widget},
};
use project_mapper_core::config::sink::{MonitorInfo, Resolution};

use crate::{
    config::{
        consts::{BORDERLESS_FULLSCREEN_MODE, EXCLUSIVE_FULLSCREEN_MODE, WINDOWED_FULLSCREEN_MODE},
        parser::ParsedAvailableConfig,
    },
    runtime_api,
    wigets::{
        elements::{
            DisplayElementConfig, ElementData, MonitorElementConfig, RegionElementConfig,
            RegionElementType, SinkElementConfig, SinkElementType, SourceElementConfig,
            SourceElementType, UiElementData, UiElementInfo, UiElementWidget,
        },
        sink::MonitorElementWidget,
    },
};
use anyhow::{Error, Result};

use super::app::{CoreApp, CoreView};

pub struct SimpleUiCore {
    config: ParsedAvailableConfig,
    uri: String,
    elements: Arc<Mutex<Vec<UiElementData>>>,
    element_infos: Vec<UiElementInfo>,
}

impl SimpleUiCore {
    pub fn new(config: ParsedAvailableConfig) -> Result<SimpleUiCore> {
        let source_data = UiElementData {
            data: ElementData::Source(SourceElementConfig {
                name: "source".to_owned(),
                id: 3,
                source: SourceElementType::URI("".to_owned()),
            }),
        };

        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        let elm = UiElementData {
            data: ElementData::Sink(SinkElementConfig {
                name: "sink1".to_owned(),
                id: 1,
                sink: SinkElementType::Monitor(MonitorElementConfig::default()),
            }),
        };
        let elm_2 = UiElementData {
            data: ElementData::Sink(SinkElementConfig {
                name: "sink2".to_owned(),
                id: 2,
                sink: SinkElementType::Monitor(MonitorElementConfig::default()),
            }),
        };
        let elm_3 = UiElementData {
            data: ElementData::Region(RegionElementConfig {
                name: "sink2".to_owned(),
                id: 2,
                region: RegionElementType::Display(DisplayElementConfig::default()),
            }),
        };

        let mut elements = vec![source_data, elm, elm_2, elm_3];
        let mut element_infos = vec![];

        for element in &mut elements {
            element_infos.push(element.info());
        }

        Ok(Self {
            config: config,
            uri: String::new(),
            elements: Arc::new(Mutex::new(elements)),
            element_infos: element_infos,
        })
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
        let mut sink_elements: Vec<&mut UiElementData> = Vec::new();
        let mut source_elements: Vec<&mut UiElementData> = Vec::new();
        let mut region_elements: Vec<&mut UiElementData> = Vec::new();

        for element in self.core.elements {
            match &mut element.data {
                ElementData::Sink(sink_config) => {
                    sink_elements.push(element);
                }
                ElementData::Source(source_config) => {
                    source_elements.push(element);
                }
                ElementData::Region(region_config) => {
                    match &mut region_config.region {
                        RegionElementType::Display(display) => {
                            display.update_elements(Some(self.core.element_infos.clone()));
                        }
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
                    let mut widget = UiElementWidget::new(source_element, self.core.config.clone());
                    source_ui.add(widget);
                }
                for region_element in region_elements {
                    let mut widget = UiElementWidget::new(region_element, self.core.config.clone());
                    region_ui.add(widget);
                }
                for sink_element in sink_elements {
                    let mut widget = UiElementWidget::new(sink_element, self.core.config.clone());
                    sink_ui.add(widget);
                }
            })
        })
        .response
    }
}
