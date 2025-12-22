use std::{
    any::Any,
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
};

use schemars::{json_schema, schema_for};
use serde::{Deserialize, Serialize};

use project_mapper_core::{
    runtime_config::output::common::OutputConfigTrait,
    types::{
        openapi::OpenAPISchema,
        video::{RefreshRate, Resolution},
    },
};
use winit::{dpi::PhysicalSize, monitor::MonitorHandle};

// struct that identifies a specific monitor and it's desired config.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MonitorConfig {
    pub name: String,
    pub resolution: Resolution,
    pub refresh_rate: RefreshRate,
}

// struct that determines what type of window we should be using
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum WindowMode {
    Windowed {},
    Borderless { name: String },
    Exclusive { config: MonitorConfig },
}

// Struct that controls the config for a window output
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WindowConfig {
    pub mode: WindowMode,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            mode: WindowMode::Windowed {},
        }
    }
}

// Implement InputConfigTrait for TestConfig
#[typetag::serde]
impl OutputConfigTrait for WindowConfig {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn OutputConfigTrait> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AvailableMonitorConfig {
    pub name: String,
    pub configs: HashSet<(Resolution, RefreshRate)>,
}

impl Hash for AvailableMonitorConfig {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state); // The 'name' field is ignored
    }
}

impl std::cmp::Eq for AvailableMonitorConfig {}

#[derive(Debug, Clone, Default)]
pub struct AvailableWindowConfig {
    pub monitors: HashSet<AvailableMonitorConfig>,
}

impl AvailableWindowConfig {
    pub fn from_monitor_handles(monitors: impl Iterator<Item = MonitorHandle>) -> Self {
        let mut monitor_tracker: HashSet<AvailableMonitorConfig> = HashSet::new();
        for monitor in monitors {
            let mut configs = HashSet::new();
            for mode in monitor.video_modes() {
                let PhysicalSize { width, height } = mode.size().into();
                let resolution = Resolution { width, height };
                let refresh_rate: RefreshRate = (mode.refresh_rate_millihertz() / 1000).into();
                configs.insert((resolution, refresh_rate));
            }
            monitor_tracker.insert(AvailableMonitorConfig {
                name: monitor.name().unwrap(),
                configs: configs,
            });
        }

        Self {
            monitors: monitor_tracker,
        }
    }

    pub fn openapi_schema(&self) -> OpenAPISchema {
        OpenAPISchema::default()
    }
}
