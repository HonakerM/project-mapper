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

use super::simple_ui::{SimpleUiApp, SimpleUiCore};

pub trait CoreView {
    fn config(self) -> Result<RuntimeConfig>;
    fn load_config(&mut self, config: RuntimeConfig) -> Result<()>;
}

pub enum CoreViews {
    SimpleUi(SimpleUiCore),
}

pub struct CoreApp {
    pub config: ParsedAvailableConfig,
    pub app: CoreViews,
}

impl CoreApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Result<CoreApp> {
        // initialize fonts
        egui_material_icons::initialize(&cc.egui_ctx);

        let available_config: json::JsonValue = runtime_api::config::get_available_config()?;
        let parsed_config = ParsedAvailableConfig::new(&available_config)?;

        Ok(CoreApp {
            config: parsed_config.clone(),
            app: CoreViews::SimpleUi(SimpleUiCore::new(parsed_config)?),
        })
    }

    pub fn header(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("my_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    let mut button = ui.button(egui_material_icons::icons::ICON_CHEVRON_RIGHT);
                    if button.clicked() {
                        let config = self.get_config().unwrap();
                        let j = serde_json::to_string(&config).unwrap();
                        println!("{}", j);
                    }
                })
            })
        });
    }

    pub fn get_config(&self) -> Result<RuntimeConfig> {
        match &self.app {
            CoreViews::SimpleUi(ui) => ui.config(),
        }
    }
}
impl App for CoreApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.header(ctx, frame);

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut ui_builder = egui::UiBuilder::new();
            ui.scope_builder(ui_builder, |ui| match &mut self.app {
                CoreViews::SimpleUi(core) => ui.add(SimpleUiApp::new(core)),
            });
        });
    }
}
