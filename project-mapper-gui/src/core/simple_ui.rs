use eframe::{
    self,
    egui::{self, TextBuffer},
};

use crate::{
    config::{
        consts::{BORDERLESS_FULLSCREEN_MODE, EXCLUSIVE_FULLSCREEN_MODE, WINDOWED_FULLSCREEN_MODE},
        parser::ParsedAvailableConfig,
    },
    runtime_api,
};
use anyhow::{Error, Result};

pub struct SimpleUI {
    config: ParsedAvailableConfig,
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
        let parsed_config = ParsedAvailableConfig::new(&available_config)?;

        let default_monitor = parsed_config
            .monitors
            .keys()
            .next()
            .or(Some(&String::from("")))
            .ok_or(Error::msg("def have value"))?
            .clone();

        Ok(Self {
            config: parsed_config,
            uri: String::new(),
            mode: WINDOWED_FULLSCREEN_MODE.to_string(),
            monitor: default_monitor,
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

        egui::ComboBox::from_label("Fullscreen Mode")
            .selected_text(format!("{mode:?}"))
            .show_ui(ui, |ui| {
                for ava_mode in config.full_screen_modes.clone() {
                    ui.selectable_value(mode, ava_mode.clone(), ava_mode.clone());
                }
            });
        ui.end_row();

        if mode == EXCLUSIVE_FULLSCREEN_MODE {
            ui.label("Monitor");
            egui::ComboBox::from_label("Monitor")
                .selected_text(format!("{monitor:?}"))
                .show_ui(ui, |ui| {
                    for ava_monitors in config.monitors.keys() {
                        ui.selectable_value(monitor, ava_monitors.clone(), ava_monitors.clone());
                    }
                });
            ui.end_row();

            if let Some(monitor_config) = config.monitors.get(monitor) {
                ui.label("Resolution");
                egui::ComboBox::from_label("Resolution")
                    .selected_text(format!("{resolution:?}"))
                    .show_ui(ui, |ui| {
                        for resolutions in monitor_config.keys() {
                            ui.selectable_value(
                                resolution,
                                resolutions.clone(),
                                resolutions.clone(),
                            );
                        }
                    });
                ui.end_row();

                if let Some(refresh_rates) = monitor_config.get(resolution) {
                    let rr_text = (*refresh_rate / 1000);

                    ui.label("Refresh Rate");
                    egui::ComboBox::from_label("Refresh Rate")
                        .selected_text(format!("{rr_text:?}"))
                        .show_ui(ui, |ui| {
                            for possible_refresh_rate in refresh_rates {
                                ui.selectable_value(
                                    refresh_rate,
                                    possible_refresh_rate.clone(),
                                    (possible_refresh_rate / 1000).to_string(),
                                );
                            }
                        });
                    ui.end_row();
                }
            }
        } else if mode == BORDERLESS_FULLSCREEN_MODE {
            ui.label("Monitor");
            egui::ComboBox::from_label("Monitor")
                .selected_text(format!("{monitor:?}"))
                .show_ui(ui, |ui| {
                    for ava_monitors in config.monitors.keys() {
                        ui.selectable_value(monitor, ava_monitors.clone(), ava_monitors.clone());
                    }
                });
            ui.end_row();
        } else if mode == WINDOWED_FULLSCREEN_MODE {
            /* Maybe do something */
        }
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
