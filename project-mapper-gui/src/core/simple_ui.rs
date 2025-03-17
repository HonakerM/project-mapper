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

pub struct SimpleUiWidget {
    config: ParsedAvailableConfig,
    uri: String,
    mode: String,
    monitor: String,
    resolution: String,
    refresh_rate: u32,
}

impl SimpleUiWidget {
    pub fn new(config: ParsedAvailableConfig) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        Self {
            config: config,
            uri: String::new(),
            mode: WINDOWED_FULLSCREEN_MODE.to_string(),
            monitor: String::from(""),
            resolution: String::from(""),
            refresh_rate: 0,
        }
    }

    fn simple_ui_grid_contents(&mut self, ui: &mut egui::Ui) {
        self.uri_source_ui(ui);
        self.monitor_sink_ui(ui, "monitor_1");
    }

    fn uri_source_ui(&mut self, ui: &mut egui::Ui) {
        let Self {
            config,
            uri,
            mode,
            monitor,
            resolution,
            refresh_rate,
        } = self;

        egui::Grid::new("soure_grid")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                ui.label("Source");
                ui.add(egui::TextEdit::singleline(uri).hint_text("URI"));
                ui.end_row();
            });
    }

    fn monitor_sink_ui(&mut self, ui: &mut egui::Ui, prefix: &str) {
        let Self {
            config,
            uri,
            mode,
            monitor,
            resolution,
            refresh_rate,
        } = self;

        egui::Grid::new(Self::get_id(prefix, "monitor_grid"))
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                ui.label("Fullscreen Mode");

                egui::ComboBox::from_id_salt(Self::get_id(prefix, "Fullscreen Mode"))
                    .selected_text(format!("{mode:?}"))
                    .show_ui(ui, |ui| {
                        for ava_mode in config.full_screen_modes.clone() {
                            ui.selectable_value(mode, ava_mode.clone(), ava_mode.clone());
                        }
                    });
                ui.end_row();

                if mode == EXCLUSIVE_FULLSCREEN_MODE || mode == BORDERLESS_FULLSCREEN_MODE {
                    ui.label("Monitor");
                    egui::ComboBox::from_id_salt(Self::get_id(prefix, "Monitor"))
                        .selected_text(format!("{monitor:?}"))
                        .show_ui(ui, |ui| {
                            for ava_monitors in config.monitors.keys() {
                                ui.selectable_value(
                                    monitor,
                                    ava_monitors.clone(),
                                    ava_monitors.clone(),
                                );
                            }
                        });
                    ui.end_row();

                    if mode == EXCLUSIVE_FULLSCREEN_MODE {
                        if let Some(monitor_config) = config.monitors.get(monitor) {
                            ui.label("Resolution");
                            egui::ComboBox::from_id_salt(Self::get_id(prefix, "Resolution"))
                                .selected_text(format!("{resolution:?}"))
                                .show_ui(ui, |ui| {
                                    let mut resolutions: Vec<Resolution> = monitor_config
                                        .keys()
                                        .map(|x| Resolution::from_json(x).expect("we can convert"))
                                        .collect::<Vec<Resolution>>();
                                    resolutions.sort_by(|a, b| b.cmp(a));
                                    for possible_resolution in resolutions {
                                        ui.selectable_value(
                                            resolution,
                                            possible_resolution.to_json(),
                                            possible_resolution.to_json(),
                                        );
                                    }
                                });
                            ui.end_row();

                            if let Some(refresh_rates) = monitor_config.get(resolution) {
                                let rr_text = (*refresh_rate / 1000);

                                ui.label("Refresh Rate");
                                egui::ComboBox::from_id_salt(Self::get_id(prefix, "Refresh Rate"))
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
                    }
                } else if mode == WINDOWED_FULLSCREEN_MODE {
                    /* Maybe do something */
                }
            });
    }

    fn ensure_good_defaults(&mut self) -> Result<()> {
        let mut default_monitor = self.monitor.clone();
        if default_monitor == "" {
            default_monitor = self
                .config
                .monitors
                .keys()
                .next()
                .or(Some(&String::from("")))
                .ok_or(Error::msg("def have value"))?
                .clone();
        }

        let mut default_resolution: String = self.resolution.clone();
        if default_monitor != "" {
            if default_resolution == ""
                || !self.config.monitors[&default_monitor].contains_key(&default_resolution)
            {
                default_resolution = self.config.monitors[&default_monitor]
                    .keys()
                    .next()
                    .unwrap_or(&String::from(""))
                    .clone();
            }
        }

        let mut default_refresh_rate: u32 = self.refresh_rate.clone();
        if default_resolution != "" {
            if default_refresh_rate == 0
                || !self.config.monitors[&default_monitor][&default_resolution]
                    .contains(&default_refresh_rate)
            {
                default_refresh_rate = self.config.monitors[&default_monitor][&default_resolution]
                    .iter()
                    .next()
                    .unwrap_or(&0)
                    .clone()
            }
        }

        self.refresh_rate = default_refresh_rate;
        self.resolution = default_resolution;
        self.monitor = default_monitor;
        Ok(())
    }

    fn get_id(prefix: &str, id: &str) -> String {
        format!("{prefix}_{id}")
    }
}
impl egui::Widget for &mut SimpleUiWidget {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        self.ensure_good_defaults();

        ui.heading("Simple UI For Project Mapper");

        let mut ui_builder = egui::UiBuilder::new();
        ui.scope_builder(ui_builder, |ui| {
            self.simple_ui_grid_contents(ui);
        })
        .response
    }
}
