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
    runtime_api::{self, runtime::RunApi},
    wigets::elements::UiElementData,
};
use anyhow::{Error, Result};

use super::{
    header::{HeaderCore, HeaderWidget},
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
    StopRuntime(),
}

pub struct CoreApp {
    pub config: ParsedAvailableConfig,
    pub app: CoreViews,
    pub app_event_receiver: Receiver<CoreEvent>,
    pub app_event_sender: Sender<CoreEvent>,
    pub current_run_api: Option<RunApi>,
    pub header_core: HeaderCore,
}

impl CoreApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Result<CoreApp> {
        // initialize fonts
        egui_material_icons::initialize(&cc.egui_ctx);

        let available_config: json::JsonValue = runtime_api::config::get_available_config()?;
        let parsed_config = ParsedAvailableConfig::new(&available_config)?;
        let (tx, rx) = std::sync::mpsc::channel();

        Ok(CoreApp {
            config: parsed_config.clone(),
            app: CoreViews::SimpleUi(SimpleUiCore::new(parsed_config)?),
            app_event_receiver: rx,
            app_event_sender: tx,
            current_run_api: None,
            header_core: HeaderCore::default(),
        })
    }

    pub fn header(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("my_panel").show(ctx, |ui| {
            ui.add(HeaderWidget::new(
                self.app_event_sender.clone(),
                self.config.clone(),
                &mut self.header_core,
            ))
        });
    }

    pub fn get_config(&mut self) -> Result<RuntimeConfig> {
        match &mut self.app {
            CoreViews::SimpleUi(ui) => ui.config(),
        }
    }
    pub fn update_config(&mut self, config: RuntimeConfig) -> Result<()> {
        match &mut self.app {
            CoreViews::SimpleUi(ui) => ui.load_config(config),
        }
    }

    pub fn refresh(&mut self) {
        self.refresh_events();
        self.refresh_process();
    }
    pub fn refresh_process(&mut self) {
        if let Some(run_api) = &mut self.current_run_api {
            if let Some(exit_status) = run_api.runtime_process.try_wait().unwrap() {
                if !run_api.killed && !exit_status.success() {
                    panic!("Runtime exited with bad status")
                }
                self.current_run_api = None;
                println!("Cleared runtime");
            }
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
                        CoreEvent::StartRuntime() => {
                            let config = self.get_config().unwrap();
                            let config = serde_json::to_string(&config).unwrap();
                            self.current_run_api =
                                Some(RunApi::construct_and_start_runtime(&config).unwrap());
                        }
                        CoreEvent::StopRuntime() => {
                            if let Some(run_api) = &mut self.current_run_api {
                                println!("Attempting to kill runtime process");
                                run_api.killed = true;
                                run_api.runtime_process.kill();
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
}
impl App for CoreApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.refresh();

        self.header(ctx, frame);

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut ui_builder = egui::UiBuilder::new();
            ui.scope_builder(ui_builder, |ui| match &mut self.app {
                CoreViews::SimpleUi(core) => ui.add(SimpleUiApp::new(core)),
            });
        });

        egui::Window::new("🔧 Settings")
            .open(&mut self.header_core.settings_window)
            .vscroll(true)
            .show(ctx, |ui| {
                ctx.settings_ui(ui);
            });
    }
}
