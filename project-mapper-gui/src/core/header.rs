use std::sync::mpsc::Sender;

use super::{
    app::{CoreApp, CoreEvent},
    simple_ui::UiEvent,
};
use crate::config::parser::ParsedAvailableConfig;
use eframe::egui::{self, Frame, Popup, Response, Ui, UiKind, Widget};
use egui::Id;

pub struct HeaderWidget {
    pub event_sender: Sender<CoreEvent>,
    pub config: ParsedAvailableConfig,
}

impl HeaderWidget {
    pub fn new(event_sender: Sender<CoreEvent>, config: ParsedAvailableConfig) -> Self {
        Self {
            event_sender: event_sender,
            config: config,
        }
    }
}
impl Widget for HeaderWidget {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                ui.add(FileHeaderWidget::new(
                    self.event_sender.clone(),
                    self.config,
                ));
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                // let mut button = ui.button(egui_material_icons::icons::ICON_CHEVRON_RIGHT);
                let mut button = ui.button(">");
                if button.clicked() {
                    let config = self.event_sender.send(CoreEvent::StartRuntime());
                }
            });
        })
        .response
    }
}

pub struct FileHeaderWidget {
    pub event_sender: Sender<CoreEvent>,
    pub config: ParsedAvailableConfig,
}

impl FileHeaderWidget {
    pub fn new(event_sender: Sender<CoreEvent>, config: ParsedAvailableConfig) -> Self {
        Self {
            event_sender: event_sender,
            config: config,
        }
    }
}

impl Widget for FileHeaderWidget {
    fn ui(self, ui: &mut Ui) -> Response {
        let main_response = Frame::new().show(ui, |ui| ui.button("File")).inner;

        Popup::menu(&main_response)
            .id(Id::new("filemenu"))
            .show(|ui| {
                if ui.button("Open").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        if let Some(str_path) = path.as_os_str().to_str() {
                            self.event_sender
                                .send(CoreEvent::LoadConfig(str_path.to_owned()));
                        }
                    }
                }
                if ui.button("Export").clicked() {
                    if let Some(path) = rfd::FileDialog::new().save_file() {
                        if let Some(str_path) = path.as_os_str().to_str() {
                            self.event_sender
                                .send(CoreEvent::ExportConfig(str_path.to_owned()));
                        }
                    }
                }
            });

        main_response
    }
}
