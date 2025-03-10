use anyhow::Result;
use std::fs;

use crate::config::runtime::RuntimeConfig;

pub fn load_config(path: &String) -> Result<RuntimeConfig> {
    let data = fs::read_to_string(path)?;

    let deserialized: RuntimeConfig = serde_json::from_str(&data).unwrap();
    Ok(deserialized)
}

pub fn export_config(config: &RuntimeConfig) -> Result<String> {
    let result = serde_json::to_string(config)?;
    Ok(result)
}
