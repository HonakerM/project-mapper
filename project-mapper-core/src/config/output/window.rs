use serde::{Deserialize, Serialize};

use crate::types::video::{RefreshRate, Resolution};

// struct that identifies a specific monitor and it's desired config.
#[derive(Serialize, Deserialize, Debug)]
pub struct MonitorConfig {
    pub name: String,
    pub resolution: Resolution,
    pub refresh_rate: RefreshRate,
}

// struct that determines what type of window we should be using
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum WindowMode {
    Windowed {},
    Borderless { name: String },
    Exclusive { config: MonitorConfig },
}

// Struct that controls the config for a window output
#[derive(Serialize, Deserialize, Debug)]
pub struct WindowConfig {
    pub mode: WindowMode,
}
