use eframe::egui::{self, Response, Ui, Widget};
use project_mapper_core::config::sink::{MonitorInfo, Resolution, SinkType};

use crate::config::{
    consts::{BORDERLESS_FULLSCREEN_MODE, EXCLUSIVE_FULLSCREEN_MODE, WINDOWED_FULLSCREEN_MODE},
    parser::ParsedAvailableConfig,
};

use super::elements::{
    ElementData, SinkElementConfig, SinkElementType, SourceElementType, UiElementData,
};

use anyhow::{Error, Result};

pub struct URIElementWidget<'a> {
    config: ParsedAvailableConfig,
    uri: &'a mut String,
}

impl<'a> URIElementWidget<'a> {
    pub fn new(
        parsed_config: ParsedAvailableConfig,
        source_data: &'a mut UiElementData,
    ) -> Result<Self> {
        match &mut source_data.data {
            ElementData::Source(sink_element) => match sink_element {
                SourceElementType::URI(value) => {
                    let mut widget = Self {
                        config: parsed_config,
                        uri: value,
                    };
                    Ok(widget)
                }
                _ => Err(Error::msg("Invalid Source Element Type")),
            },
            _ => Err(Error::msg("Invalid Source Element Type")),
        }
    }
}

impl<'a> Widget for URIElementWidget<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        egui::Grid::new("soure_grid")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                ui.label("Source");
                ui.add(egui::TextEdit::singleline(self.uri).hint_text("URI"));
                ui.end_row();
            })
            .response
    }
}
