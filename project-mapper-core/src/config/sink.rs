use std::{cmp::Ordering, mem};

use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

pub type RefreshRate = u32;

pub type ResolutionJson = String;

#[derive(Serialize, Deserialize, Clone, Hash, PartialEq, Debug)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Resolution {
    pub fn to_json(&self) -> ResolutionJson {
        format!("{}x{}", self.width, self.height)
    }
    pub fn from_json(res: &ResolutionJson) -> Result<Resolution> {
        let options: Vec<&str> = res.split("x").collect();
        let width = options.get(0).ok_or(Error::msg("no width"))?;
        let height = options.get(1).ok_or(Error::msg("no height"))?;

        let width = width.parse::<u32>()?;
        let height = height.parse::<u32>()?;
        Ok(Resolution {
            width: width,
            height: height,
        })
    }
}
impl Eq for Resolution {}
impl Ord for Resolution {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.width, self.height).cmp(&(other.width, other.height))
    }
}
impl PartialOrd for Resolution {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MonitorInfo {
    pub name: String,
    pub resolution: ResolutionJson,
    pub refresh_rate_hz: RefreshRate,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum FullScreenMode {
    Windowed {},
    Borderless { name: String },
    Exclusive { info: MonitorInfo },
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum SinkType {
    OpenGLWindow { full_screen: FullScreenMode },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SinkConfig {
    pub name: String,
    pub id: u32,
    pub sink: SinkType,
}
