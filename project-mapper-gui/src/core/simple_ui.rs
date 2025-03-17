use eframe::{
    self, App,
    egui::{self, TextBuffer},
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
        },
        sink::MonitorElementWidget,
    },
};
use anyhow::{Error, Result};

use super::app::CoreApp;

pub struct SimpleUiApp {
    app: CoreApp,
    uri: String,
    element: ElementData,
    elements: Vec<UiElementData>,
}

impl SimpleUiApp {
    pub fn new() -> Result<SimpleUiApp> {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        Ok(Self {
            app: CoreApp::new()?,
            uri: String::new(),
            elements: Vec::new(),
            element: ElementData::Sink(SinkElementConfig {
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
        })
    }

    fn simple_ui_grid_contents(&mut self, ui: &mut egui::Ui) {
        ui.heading("Simple UI For Project Mapper");

        self.uri_source_ui(ui);

        let mut widget =
            MonitorElementWidget::new(self.app.config.clone(), &mut self.element).expect("uh oh");
        ui.add(&mut widget);
    }

    fn uri_source_ui(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("soure_grid")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                ui.label("Source");
                ui.add(egui::TextEdit::singleline(&mut self.uri).hint_text("URI"));
                ui.end_row();
            });
    }
}
impl App for SimpleUiApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.app.header(ctx, frame);

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut ui_builder = egui::UiBuilder::new();
            ui.scope_builder(ui_builder, |ui| {
                self.simple_ui_grid_contents(ui);
            });
        });
    }
}
