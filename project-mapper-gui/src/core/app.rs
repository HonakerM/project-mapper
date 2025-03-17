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

pub struct CoreApp {
    pub config: ParsedAvailableConfig,
}

impl CoreApp {
    pub fn new() -> Result<CoreApp> {
        let available_config: json::JsonValue = runtime_api::config::get_available_config()?;
        let parsed_config = ParsedAvailableConfig::new(&available_config)?;

        Ok(CoreApp {
            config: parsed_config,
        })
    }

    pub fn header(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("my_panel").show(ctx, |ui| {
            ui.label("Hello World! From `TopBottomPanel`, that must be before `CentralPanel`!");
        });
    }
}
