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
use project_mapper_core::config::sink::{RefreshRate, Resolution, ResolutionJson};
use raw_window_handle::HasWindowHandle;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::monitor::{MonitorHandle, VideoModeHandle};
use winit::window::{Window, WindowId};

// internal for config
pub struct MonitorData {
    pub name: String,
    pub monitor: MonitorHandle,
    pub mode_lookup: HashMap<ResolutionJson, HashMap<RefreshRate, VideoModeHandle>>,
}

pub fn gather_monitor_info(event_loop: &ActiveEventLoop) -> HashMap<String, MonitorData> {
    let mut monitor_map = HashMap::new();
    for monitor in event_loop.available_monitors() {
        let mut resolution_map: HashMap<ResolutionJson, HashMap<RefreshRate, VideoModeHandle>> =
            HashMap::new();
        for monitor_handle in monitor.video_modes() {
            let size = monitor.size();
            let resolution = Resolution {
                height: size.height,
                width: size.width,
            }
            .to_json();

            if !resolution_map.contains_key(&resolution) {
                resolution_map.insert(resolution.clone(), HashMap::new());
            }

            let frequency_map = resolution_map.get_mut(&resolution).expect("we just added");

            let refresh_rate_mhz: RefreshRate =
                RefreshRate::from(monitor_handle.refresh_rate_millihertz());
            frequency_map.insert(refresh_rate_mhz, monitor_handle);
        }
        let monitor_name = sanitize_monitor_name(monitor.name().expect("we have a name"));

        let monitor_data = MonitorData {
            name: monitor_name.clone(),
            monitor: monitor,
            mode_lookup: resolution_map,
        };
        monitor_map.insert(monitor_name, monitor_data);
    }
    monitor_map
}

fn sanitize_monitor_name(monitor_name: String) -> String {
    monitor_name.replace("\\", "")
}
