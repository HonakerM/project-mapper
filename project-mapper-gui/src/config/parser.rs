use std::{collections::HashMap, hash::Hash};

use anyhow::{Error, Result};

pub struct ParsedAvailableConfig {
    pub full_screen_modes: Vec<String>,
    pub monitors: HashMap<String, HashMap<String, Vec<u32>>>,
}

impl ParsedAvailableConfig {
    pub fn new(config: &json::JsonValue) -> Result<ParsedAvailableConfig> {
        let modes = ParsedAvailableConfig::extract_fullscreen_types(config)?;
        let monitors = ParsedAvailableConfig::extract_monitor_info(config)?;
        Ok(ParsedAvailableConfig {
            full_screen_modes: modes,
            monitors: monitors,
        })
    }
    pub fn extract_fullscreen_types(config: &json::JsonValue) -> Result<Vec<String>> {
        let mut modes = vec![];
        for data in config["sinks"].members() {
            if data["type"] != "OpenGLWindow" {
                continue;
            }

            for mode in data["full_screen_modes"].members() {
                let mode_string = mode["type"].as_str().unwrap();
                modes.push(String::from(mode_string));
            }
        }
        Ok(modes)
    }

    pub fn extract_monitor_info(
        config: &json::JsonValue,
    ) -> Result<HashMap<String, HashMap<String, Vec<u32>>>> {
        let mut monitor_hashmap: HashMap<String, HashMap<String, Vec<u32>>> = HashMap::new();
        for data in config["sinks"].members() {
            if data["type"] != "OpenGLWindow" {
                continue;
            }

            for mode in data["full_screen_modes"].members() {
                let mode_string = mode["type"].as_str().unwrap();

                if mode_string != "Exclusive" {
                    continue;
                }

                for (monitors, resolution_data) in mode["monitor_configs"].entries() {
                    let mut refresh_rate_hashmap = HashMap::new();
                    for (resolution, refresh_rates) in resolution_data.entries() {
                        let mut u32_refresh_rate = Vec::new();
                        for rr in refresh_rates.members() {
                            let u32_rate = rr.as_u32().ok_or(Error::msg("uh oh"))?;
                            u32_refresh_rate.push(u32_rate);
                        }
                        refresh_rate_hashmap.insert(String::from(resolution), u32_refresh_rate);
                    }

                    monitor_hashmap.insert(String::from(monitors), refresh_rate_hashmap);
                }
            }
        }
        Ok(monitor_hashmap)
    }
}
