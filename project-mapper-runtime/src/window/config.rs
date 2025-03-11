use std::collections::HashMap;
use std::sync::mpsc;

use gst::prelude::PadExt;
use project_mapper_core::config::events;
use project_mapper_core::config::options::{
    BorderlessOptions, ExclusiveOptions, FullscreenOptions, SinkTypeOptions, WindowOptions,
};
use project_mapper_core::config::sink::{RefreshRate, ResolutionJson};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

use super::Message;

pub struct ConfigHandler {
    event_sender: mpsc::Sender<events::OptionEvent>,
}

impl ConfigHandler {
    pub fn new(event_sender: mpsc::Sender<events::OptionEvent>) -> ConfigHandler {
        ConfigHandler { event_sender }
    }
}

impl ApplicationHandler<Message> for ConfigHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let monitor_map = super::utils::gather_monitor_info(event_loop);

        let mut monitor_configs: HashMap<String, HashMap<ResolutionJson, Vec<RefreshRate>>> =
            HashMap::new();
        for (monitor_name, monitor_data) in &monitor_map {
            let mut resolution_map = HashMap::new();
            for (resolution, refresh_rate_map) in &monitor_data.mode_lookup {
                let mut refresh_rates: Vec<RefreshRate> = Vec::new();
                for refresh_rate in refresh_rate_map.keys() {
                    refresh_rates.push(refresh_rate.clone())
                }
                resolution_map.insert(resolution.clone(), refresh_rates);
            }
            monitor_configs.insert(monitor_name.clone(), resolution_map);
        }

        let mut monitor_names = Vec::new();
        for name in monitor_configs.keys() {
            monitor_names.push(name.clone());
        }

        let fullscreen_options = vec![
            FullscreenOptions::Exclusive(ExclusiveOptions {
                monitor_configs: monitor_configs,
            }),
            FullscreenOptions::Windowed(WindowOptions {}),
            FullscreenOptions::Borderless(BorderlessOptions {
                monitors: monitor_names,
            }),
        ];

        let sink_options = SinkTypeOptions::OpenGLWindow {
            full_screen_modes: fullscreen_options,
        };

        self.event_sender
            .send(events::OptionEvent::OpenGLWindowOptions(sink_options));

        // close event loop after sending configs
        event_loop.exit();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {}

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: Message) {}
}
