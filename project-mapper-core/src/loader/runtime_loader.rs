use anyhow::Result;
use std::fs;

use crate::runtime_config::RuntimeConfig;

// load a runtime config from a file
pub fn load_config_file(path: &String) -> Result<RuntimeConfig> {
    let data = fs::read_to_string(path)?;

    load_config_json(&data)
}

// load a runtime config from a raw json
pub fn load_config_json(data: &String) -> Result<RuntimeConfig> {
    let deserialized: RuntimeConfig = serde_json::from_str(data).unwrap();
    Ok(deserialized)
}

// export a runtime config to a file
pub fn export_config_file(path: &String, config: &RuntimeConfig) -> Result<()> {
    let serialized_config = export_config_json(config)?;

    fs::write(path, serialized_config)?;

    Ok(())
}

// export a runtime config to a raw json string
pub fn export_config_json(config: &RuntimeConfig) -> Result<String> {
    let result = serde_json::to_string(config)?;
    Ok(result)
}