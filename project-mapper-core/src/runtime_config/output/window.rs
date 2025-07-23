use std::any::Any;

use serde::{Deserialize, Serialize};

use crate::{
    runtime_config::output::common::OutputConfigTrait,
    types::video::{RefreshRate, Resolution},
};

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
