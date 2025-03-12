use eframe::{
    self,
    egui::{self, TextBuffer},
};
use project_mapper_core::config::options::AvailableConfig;

use crate::runtime_api;
use anyhow::Result;

pub struct SimpleUI {
    config: AvailableConfig,
    uri: String,
    monitor: String,
    resolution: String,
    refresh_rate: u32,
}

impl SimpleUI {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Result<Self> {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        let available_config = runtime_api::config::get_available_config()?;

        Ok(Self {
            config: available_config,
            uri: String::new(),
            monitor: String::new(),
            resolution: String::new(),
            refresh_rate: 0,
        })
    }

    fn simple_ui_grid_contents(&mut self, ui: &mut egui::Ui) {
        ui.label("Source");
        ui.add(egui::TextEdit::singleline(&mut self.uri).hint_text("URI"));
        ui.end_row();
    }
}
impl eframe::App for SimpleUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Simple UI For Project Mapper");

            let mut ui_builder = egui::UiBuilder::new();
            ui.scope_builder(ui_builder, |ui| {
                ui.multiply_opacity(1.0);

                egui::Grid::new("my_grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        self.simple_ui_grid_contents(ui);
                    });
            });
        });
    }
}
