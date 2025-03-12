use std::process::Command;

use project_mapper_core::config::{env::EnvConfig, options::AvailableConfig};

use anyhow::Result;

pub fn get_available_config() -> Result<AvailableConfig> {
    let env_config = EnvConfig::get_config();

    let command_output = Command::new(env_config.runtime_bin)
        .arg("get-available-config")
        .output()?;

    let config: AvailableConfig = serde_json::from_slice(&command_output.stdout)?;
    Ok(config)
}
