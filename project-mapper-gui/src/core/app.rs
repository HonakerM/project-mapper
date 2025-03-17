use eframe::{
    self,
    egui::{self, TextBuffer},
};
use project_mapper_core::config::sink::Resolution;

use crate::{
    config::{
        consts::{BORDERLESS_FULLSCREEN_MODE, EXCLUSIVE_FULLSCREEN_MODE, WINDOWED_FULLSCREEN_MODE},
        parser::ParsedAvailableConfig,
    },
    runtime_api,
};
use anyhow::{Error, Result};

use super::simple_ui::SimpleUiWidget;

pub struct CoreApp {
    config: ParsedAvailableConfig,
}

impl CoreApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Result<CoreApp> {
        let available_config: json::JsonValue = runtime_api::config::get_available_config()?;
        let parsed_config = ParsedAvailableConfig::new(&available_config)?;

        Ok(CoreApp {
            config: parsed_config,
        })
    }
}

impl eframe::App for CoreApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("my_panel").show(ctx, |ui| {
            ui.label("Hello World! From `TopBottomPanel`, that must be before `CentralPanel`!");
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut simple_ui = SimpleUiWidget::new(self.config.clone());
            ui.add(&mut simple_ui);
        });
    }
}
