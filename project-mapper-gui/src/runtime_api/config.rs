use std::process::Command;

use crate::config::{EnvConfig};

use anyhow::Result;

pub fn get_available_config() -> Result<json::JsonValue> {
    let env_config = EnvConfig::get_config();

    let command_output = Command::new(env_config.runtime_bin)
        .arg("get-available-config")
        .output()?;

    let output = String::from_utf8(
        command_output.stdout
    )?;
    let config = json::parse(&output)?;
    Ok(config)
}
