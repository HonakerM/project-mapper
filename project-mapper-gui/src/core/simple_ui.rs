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
            ElementData, MonitorElementConfig, SinkElementConfig, SinkElementType, UiElementData,
            UiElementWidget,
        },
        sink::MonitorElementWidget,
    },
};
use anyhow::{Error, Result};

use super::app::CoreApp;

pub struct SimpleUiCore {
    config: ParsedAvailableConfig,
    uri: String,
    elements: Vec<UiElementData>,
}

impl SimpleUiCore {
    pub fn new(config: ParsedAvailableConfig) -> Result<SimpleUiCore> {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        let elm = UiElementData {
            data: ElementData::Sink(SinkElementConfig {
                name: "sink1".to_owned(),
                id: 1,
                sink: SinkElementType::Monitor(MonitorElementConfig {
                    mode: WINDOWED_FULLSCREEN_MODE.to_owned(),
                    monitor: MonitorInfo {
                        name: "".to_owned(),
                        resolution: "".to_owned(),
                        refresh_rate_hz: 0,
                    },
                }),
            }),
        };
        let elm_2 = UiElementData {
            data: ElementData::Sink(SinkElementConfig {
                name: "sink2".to_owned(),
                id: 2,
                sink: SinkElementType::Monitor(MonitorElementConfig {
                    mode: WINDOWED_FULLSCREEN_MODE.to_owned(),
                    monitor: MonitorInfo {
                        name: "".to_owned(),
                        resolution: "".to_owned(),
                        refresh_rate_hz: 0,
                    },
                }),
            }),
        };

        Ok(Self {
            config: config,
            uri: String::new(),
            elements: vec![elm, elm_2],
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

        for element in &mut self.core.elements {
            match &mut element.data {
                ElementData::Sink(sink_config) => {
                    sink_elements.push(element);
                }
                _ => {}
            }
        }

        let mut ui_builder = egui::UiBuilder::new();
        ui.scope_builder(ui_builder, |ui| {
            ui.heading("Simple UI For Project Mapper");

            egui::Grid::new("soure_grid")
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Source");
                    ui.add(egui::TextEdit::singleline(&mut self.core.uri).hint_text("URI"));
                    ui.end_row();
                });

            for sink_element in sink_elements {
                let mut widget = UiElementWidget::new(sink_element, self.core.config.clone());
                ui.add(widget);
            }
        })
        .response
    }
}
