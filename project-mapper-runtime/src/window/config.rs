use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::mpsc;

use crate::opengl::{self, gl};
use anyhow::{Context, Error, Result};
use glutin::config::{GetGlConfig, GlConfig};
use glutin::context::AsRawContext;
use glutin::display::{AsRawDisplay, GetGlDisplay};
use glutin::prelude::{GlDisplay, NotCurrentGlContext, PossiblyCurrentGlContext};
use glutin::surface::GlSurface;
use glutin_winit::GlWindow;
use gst::prelude::{ElementExt, GstObjectExt, PadExt, PadExtManual};
use gst::{PadProbeReturn, PadProbeType, QueryViewMut, element_error};
use gst_gl::GLVideoFrameExt;
use gst_gl::prelude::GLContextExt;
use gst_video::VideoFrameExt;
use project_mapper_core::config::events;
use project_mapper_core::config::options::{
    BorderlessOptions, ExclusiveOptions, FullscreenOptions, WindowOptions,
};
use project_mapper_core::config::sink::{RefreshRate, Resolution, ResolutionJson};
use raw_window_handle::HasWindowHandle;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::monitor::{MonitorHandle, VideoModeHandle};
use winit::window::{Window, WindowId};

use super::Message;
use super::utils::MonitorData;

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

        let fullscreen_options = FullscreenOptions {
            exclusive: ExclusiveOptions {
                type_name: String::from("Exclusive"),
                monitor_configs: monitor_configs,
            },
            windowed: WindowOptions {
                type_name: String::from("Windowed"),
            },
            borderless: BorderlessOptions {
                type_name: String::from("Borderless"),
                monitors: monitor_names,
            },
        };

        self.event_sender
            .send(events::OptionEvent::OpenGLWindowOptions(fullscreen_options));

        // close event loop after sending configs
        event_loop.exit();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {}

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: Message) {}
}
