use std::sync::mpsc::Sender;

use super::{
    app::{CoreApp, CoreEvent},
    simple_ui::UiEvent,
};
use crate::config::parser::ParsedAvailableConfig;
use eframe::egui::{self, Frame, Popup, Response, Ui, UiKind, Widget};
use egui::Id;

pub struct HeaderCore {
    pub settings_window: bool,
}
impl HeaderCore {
    pub fn default() -> Self {
        Self {
            settings_window: false,
        }
    }
}
pub struct HeaderWidget<'a> {
    pub core: &'a mut HeaderCore,
    pub event_sender: Sender<CoreEvent>,
    pub config: ParsedAvailableConfig,
}

impl<'a> HeaderWidget<'a> {
    pub fn new(
        event_sender: Sender<CoreEvent>,
        config: ParsedAvailableConfig,
        core: &'a mut HeaderCore,
    ) -> Self {
        Self {
            core: core,
            event_sender: event_sender,
            config: config,
        }
    }
}
impl<'a> Widget for HeaderWidget<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                ui.add(FileHeaderWidget::new(
                    self.event_sender.clone(),
                    self.config.clone(),
                ));
                ui.add(ViewHeaderWidget::new(
                    self.event_sender.clone(),
                    self.config,
                    &mut self.core.settings_window,
                ));
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if ui
                    .button(egui_material_icons::icons::ICON_CHEVRON_RIGHT)
                    .clicked()
                {
                    let config = self.event_sender.send(CoreEvent::StartRuntime());
                }
                if ui.button(egui_material_icons::icons::ICON_CANCEL).clicked() {
                    let config = self.event_sender.send(CoreEvent::StopRuntime());
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
pub struct ViewHeaderWidget<'a> {
    pub event_sender: Sender<CoreEvent>,
    pub config: ParsedAvailableConfig,
    pub settings: &'a mut bool,
}

impl<'a> ViewHeaderWidget<'a> {
    pub fn new(
        event_sender: Sender<CoreEvent>,
        config: ParsedAvailableConfig,
        settings_window: &'a mut bool,
    ) -> Self {
        Self {
            event_sender: event_sender,
            config: config,
            settings: settings_window,
        }
    }
}

impl<'a> Widget for ViewHeaderWidget<'a> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let main_response = Frame::new().show(ui, |ui| ui.button("View")).inner;

        Popup::menu(&main_response)
            .id(Id::new("viewmenu"))
            .show(|ui| {
                egui::widgets::global_theme_preference_switch(ui);
                ui.separator();
                if ui.button("🔧 Settings").clicked() {
                    *self.settings = !*self.settings;
                }
            });
        main_response
    }
}
