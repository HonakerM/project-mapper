use eframe::{
    self,
    egui::{self, TextBuffer},
};

use crate::runtime_api;
use anyhow::Result;

pub struct SimpleUI {
    config: json::JsonValue,
    uri: String,
    mode: String,
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

        let available_config: json::JsonValue = runtime_api::config::get_available_config()?;

        Ok(Self {
            config: available_config,
            uri: String::new(),
            mode: String::new(),
            monitor: String::new(),
            resolution: String::new(),
            refresh_rate: 0,
        })
    }

    fn simple_ui_grid_contents(&mut self, ui: &mut egui::Ui) {
        let Self {
            config,
            uri,
            mode,
            monitor,
            resolution,
            refresh_rate,
        } = self;

        ui.label("Source");
        ui.add(egui::TextEdit::singleline(uri).hint_text("URI"));
        ui.end_row();


        ui.label("Fullscreen Mode");
        
        egui::ComboBox::from_label("")
            .selected_text(format!("{mode:?}"))
            .show_ui(ui, |ui| {
            });
        ui.end_row();

        ui.label("Fullscreen Mode");


    }
}
impl eframe::App for SimpleUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Simple UI For Project Mapper");

            let mut ui_builder = egui::UiBuilder::new();
            ui.scope_builder(ui_builder, |ui| {
                egui::Grid::new("my_grid")
                    .num_columns(2)
                    .striped(true)
                    .show(ui, |ui| {
                        self.simple_ui_grid_contents(ui);
                    });
            });
        });
    }
}
