use std::{
    fs,
    sync::mpsc::{Receiver, Sender},
};

use eframe::{
    self, App,
    egui::{self, TextBuffer, Widget},
};
use project_mapper_core::config::{runtime::RuntimeConfig, sink::Resolution};

use crate::{
    config::{
        consts::{BORDERLESS_FULLSCREEN_MODE, EXCLUSIVE_FULLSCREEN_MODE, WINDOWED_FULLSCREEN_MODE},
        parser::ParsedAvailableConfig,
    },
    runtime_api,
    wigets::elements::UiElementData,
};
use anyhow::{Error, Result};

use super::{
    header::HeaderWidget,
    simple_ui::{SimpleUiApp, SimpleUiCore},
};

pub trait CoreView {
    fn config(self) -> Result<RuntimeConfig>;
    fn load_config(self, config: RuntimeConfig) -> Result<()>;
}

pub enum CoreViews {
    SimpleUi(SimpleUiCore),
}

pub enum CoreEvent {
    LoadConfig(String),
    ExportConfig(String),
    StartRuntime(),
}

pub struct CoreApp {
    pub config: ParsedAvailableConfig,
    pub app: CoreViews,
    pub app_event_receiver: Receiver<CoreEvent>,
    pub app_event_sender: Sender<CoreEvent>,
}

impl CoreApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Result<CoreApp> {
        // initialize fonts
        //egui_material_icons::initialize(&cc.egui_ctx);

        let available_config: json::JsonValue = runtime_api::config::get_available_config()?;
        let parsed_config = ParsedAvailableConfig::new(&available_config)?;
        let (tx, rx) = std::sync::mpsc::channel();

        Ok(CoreApp {
            config: parsed_config.clone(),
            app: CoreViews::SimpleUi(SimpleUiCore::new(parsed_config)?),
            app_event_receiver: rx,
            app_event_sender: tx,
        })
    }

    pub fn header(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("my_panel").show(ctx, |ui| {
            ui.add(HeaderWidget::new(
                self.app_event_sender.clone(),
                self.config.clone(),
            ))
        });
    }

    pub fn get_config(&self) -> Result<RuntimeConfig> {
        match &self.app {
            CoreViews::SimpleUi(ui) => ui.config(),
        }
    }
    pub fn update_config(&mut self, config: RuntimeConfig) -> Result<()> {
        match &mut self.app {
            CoreViews::SimpleUi(ui) => ui.load_config(config),
        }
    }

    pub fn refresh_events(&mut self) {
        loop {
            match self.app_event_receiver.try_recv() {
                Ok(msg) => {
                    // add to buffer/queue but process all when buffer.len() = 100
                    match msg {
                        CoreEvent::ExportConfig(path) => {
                            let config = self.get_config().unwrap();
                            let config = serde_json::to_string(&config).unwrap();
                            fs::write(path, config);
                        }
                        CoreEvent::LoadConfig(path) => {
                            let config = fs::read(path).unwrap();
                            let config: RuntimeConfig = serde_json::from_slice(&config).unwrap();
                            self.update_config(config);
                        }
                        CoreEvent::StartRuntime() => {}
                    }
                }
                Err(e) => {
                    break;
                }
            }
        }
    }
}
impl App for CoreApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.refresh_events();

        self.header(ctx, frame);

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut ui_builder = egui::UiBuilder::new();
            ui.scope_builder(ui_builder, |ui| match &mut self.app {
                CoreViews::SimpleUi(core) => ui.add(SimpleUiApp::new(core)),
            });
        });
    }
}
