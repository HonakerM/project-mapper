use eframe::egui::{self, Response, Ui, Widget};
use project_mapper_core::config::sink::{MonitorInfo, Resolution, SinkType};

use crate::config::{
    consts::{BORDERLESS_FULLSCREEN_MODE, EXCLUSIVE_FULLSCREEN_MODE, WINDOWED_FULLSCREEN_MODE},
    parser::ParsedAvailableConfig,
};

use super::elements::{
    ElementData, RegionElementType, SinkElementType, SourceElementType, UiElementData,
    UiElementInfo,
};

use anyhow::{Error, Result};

pub struct DisplayElementWidget<'a> {
    src_infos: Vec<UiElementInfo>,
    sink_infos: Vec<UiElementInfo>,
    config: ParsedAvailableConfig,
    src_info: &'a mut Option<UiElementInfo>,
    sink_info: &'a mut Option<UiElementInfo>,
}

impl<'a> DisplayElementWidget<'a> {
    pub fn new(
        parsed_config: ParsedAvailableConfig,
        region_data: &'a mut UiElementData,
    ) -> Result<Self> {
        match &mut region_data.data {
            ElementData::Region(sink_element) => match sink_element {
                RegionElementType::Display(display) => {
                    let mut widget = Self {
                        src_infos: vec![],
                        sink_infos: vec![],
                        config: parsed_config,
                        src_info: &mut display.source,
                        sink_info: &mut display.sink,
                    };
                    if let Some(element_infos) = &mut display.element_infos {
                        for info in element_infos {
                            match info {
                                UiElementInfo::Source { id, name } => {
                                    widget.src_infos.push(info.clone());
                                }
                                UiElementInfo::Sink { id, name } => {
                                    widget.sink_infos.push(info.clone());
                                }
                                _ => {}
                            }
                        }
                    }

                    widget.validate_config();

                    Ok(widget)
                }
                _ => Err(Error::msg("Invalid Region Element Type")),
            },
            _ => Err(Error::msg("Invalid Region Element Type")),
        }
    }

    pub fn validate_config(&mut self) {
        let mut valid_src = false;
        let mut first_src_info = None;
        for src_info in &self.src_infos {
            if let None = first_src_info {
                first_src_info = Some(src_info.clone())
            }
            if let Some(current_src_info) = self.src_info {
                if current_src_info.id() == src_info.id() {
                    current_src_info.set_name(src_info.name());
                    valid_src = true;
                }
            }
        }
        if !valid_src {
            if let Some(first_src) = first_src_info {
                *self.src_info = Some(first_src);
            } else {
                *self.src_info = None
            }
        }

        let mut valid_sink = false;
        let mut first_sink_info = None;
        for sink_info in &self.sink_infos {
            if let None = first_sink_info {
                first_sink_info = Some(sink_info.clone())
            }
            if let Some(current_sink_info) = self.sink_info {
                if current_sink_info.id() == sink_info.id() {
                    current_sink_info.set_name(sink_info.name());
                    valid_sink = true;
                }
            }
        }
        if !valid_sink {
            if let Some(first_sink) = first_sink_info {
                *self.sink_info = Some(first_sink);
            } else {
                *self.sink_info = None
            }
        }
    }
}

impl<'a> Widget for DisplayElementWidget<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        egui::Grid::new("soure_grid")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                ui.label("Source");

                let mut src_id_text = "".to_string();
                if let Some(info) = self.src_info {
                    src_id_text = info.name();
                }

                egui::ComboBox::from_id_salt("Source")
                    .selected_text(src_id_text)
                    .show_ui(ui, |ui| {
                        for info in self.src_infos {
                            let name = info.name().clone();
                            ui.selectable_value(self.src_info, Some(info), name);
                        }
                    });
                ui.end_row();

                let mut sink_id_text = "".to_string();
                if let Some(info) = self.sink_info {
                    sink_id_text = info.name();
                }

                ui.label("Sink");
                egui::ComboBox::from_id_salt("Sink")
                    .selected_text(sink_id_text)
                    .show_ui(ui, |ui| {
                        for info in self.sink_infos {
                            let name = info.name().clone();
                            ui.selectable_value(self.sink_info, Some(info), name);
                        }
                    });
                ui.end_row();
            })
            .response
    }
}
