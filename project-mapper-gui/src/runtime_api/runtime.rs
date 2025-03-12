use std::{
    fs::{self, File},
    io::Write,
    process::{Child, Command},
    thread::sleep,
    time::Duration,
};

use anyhow::{Error, Result};
use project_mapper_core::config::{
    env::EnvConfig, options::AvailableConfig, runtime::RuntimeConfig,
};
use tempdir::TempDir;

pub struct RunApi {
    pub runtime_process: Child,
    pub temp_dir: TempDir,
}

impl RunApi {
    pub fn construct_and_start_runtime(config: RuntimeConfig) -> Result<RunApi> {
        let env_config = EnvConfig::get_config();

        let config_json = serde_json::to_string(&config)?;

        let temp_dir = TempDir::new("temp_config")?;

        let file_path = temp_dir.path().join("config.yml");
        fs::write(&file_path, config_json)?;
        let file_path = file_path.to_str().ok_or(Error::msg("no temp file path"))?;

        let mut command_output = Command::new(env_config.runtime_bin)
            .arg("run")
            .arg(file_path)
            .spawn()?;

        Ok(RunApi {
            runtime_process: command_output,
            temp_dir: temp_dir,
        })
    }
}
