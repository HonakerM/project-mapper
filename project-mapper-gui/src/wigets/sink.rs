use eframe::egui::{self, Response, Ui, Widget};
use project_mapper_core::config::sink::{MonitorInfo, Resolution, SinkType};

use crate::config::{
    consts::{BORDERLESS_FULLSCREEN_MODE, EXCLUSIVE_FULLSCREEN_MODE, WINDOWED_FULLSCREEN_MODE},
    parser::ParsedAvailableConfig,
};

use super::elements::{ElementData, SinkElementConfig, SinkElementType};

use anyhow::{Error, Result};

pub struct MonitorElementWidget<'a> {
    config: ParsedAvailableConfig,
    mode: &'a mut String,
    monitor: &'a mut MonitorInfo,
}

impl<'a> MonitorElementWidget<'a> {
    pub fn new(
        parsed_config: ParsedAvailableConfig,
        sink_data: &'a mut SinkElementConfig,
    ) -> Result<Self> {
        match &mut sink_data.sink {
            SinkElementType::Monitor(monitor) => {
                let mut widget = Self {
                    config: parsed_config,
                    mode: &mut monitor.mode,
                    monitor: &mut monitor.monitor,
                };
                widget.ensure_good_selection();
                Ok(widget)
            }
        }
    }

    fn ensure_good_selection(&mut self) -> Result<()> {
        let mut default_monitor = self.monitor.name.clone();
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

        let mut default_resolution: String = self.monitor.resolution.clone();
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

        let mut default_refresh_rate: u32 = self.monitor.refresh_rate_hz.clone();
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

        self.monitor.refresh_rate_hz = default_refresh_rate;
        self.monitor.resolution = default_resolution;
        self.monitor.name = default_monitor;
        Ok(())
    }
}

impl<'a> Widget for MonitorElementWidget<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let config = &self.config;
        let mut monitor = &mut self.monitor.name;
        let mut resolution = &mut self.monitor.resolution;
        let mut refresh_rate = &mut self.monitor.refresh_rate_hz;

        egui::Grid::new("monitor_grid")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                ui.label("Fullscreen Mode");

                let current_text = self.mode.clone();
                egui::ComboBox::from_id_salt("Fullscreen Mode")
                    .selected_text(format!("{current_text:?}"))
                    .show_ui(ui, |ui| {
                        for ava_mode in config.full_screen_modes.clone() {
                            ui.selectable_value(self.mode, ava_mode.clone(), ava_mode.clone());
                        }
                    });
                ui.end_row();

                if self.mode == EXCLUSIVE_FULLSCREEN_MODE || self.mode == BORDERLESS_FULLSCREEN_MODE
                {
                    ui.label("Monitor");
                    egui::ComboBox::from_id_salt("Monitor")
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

                    if self.mode == EXCLUSIVE_FULLSCREEN_MODE {
                        if let Some(monitor_config) = config.monitors.get(monitor) {
                            ui.label("Resolution");
                            egui::ComboBox::from_id_salt("Resolution")
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
                                egui::ComboBox::from_id_salt("Refresh Rate")
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
                } else if self.mode == WINDOWED_FULLSCREEN_MODE {
                    /* Maybe do something */
                }
            })
            .response
    }
}
