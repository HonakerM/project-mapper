use anyhow::{Error, Result, anyhow};
use winit::{
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    monitor::{MonitorHandle, VideoMode, VideoModeHandle},
};

use crate::output::window::config::MonitorConfig;

pub fn get_monitor_by_name(event_loop: &ActiveEventLoop, name: String) -> Result<MonitorHandle> {
    for monitor in event_loop.available_monitors() {
        if let Some(monitor_name) = monitor.name() {
            if monitor_name == name {
                return Ok(monitor);
            }
        }
    }
    Err(Error::msg(format!(
        "Unable to find monitor with name {}",
        name
    )))
}

pub fn get_video_mode_for_config(
    event_loop: &ActiveEventLoop,
    config: &MonitorConfig,
) -> Result<VideoModeHandle> {
    // Get all available monitors
    let monitors: Vec<MonitorHandle> = event_loop.available_monitors().collect();

    // Find the monitor matching the config name
    let monitor = monitors
        .into_iter()
        .find(|m| m.name().map_or(false, |n| n == config.name))
        .ok_or_else(|| anyhow!("Monitor '{}' not found", config.name))?;

    // Find a matching video mode
    let video_mode = monitor
        .video_modes()
        .find(|mode| {
            let size = mode.size();
            let refresh = mode.refresh_rate_millihertz() / 1000;
            size.width == config.resolution.width
                && size.height == config.resolution.height
                && refresh == config.refresh_rate
        })
        .ok_or_else(|| {
            anyhow!(
                "No matching video mode for resolution {}x{} @ {}Hz on monitor '{}'",
                config.resolution.width,
                config.resolution.height,
                config.refresh_rate,
                config.name
            )
        })?;

    Ok(video_mode)
}
