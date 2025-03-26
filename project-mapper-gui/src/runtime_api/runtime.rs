use std::{
    fs::{self, File},
    io::Write,
    process::{Child, Command, Stdio},
    thread::sleep,
    time::Duration,
};

use crate::config::EnvConfig;
use anyhow::{Error, Result};
use tempdir::TempDir;

pub struct RunApi {
    pub runtime_process: Child,
    pub killed: bool,
}

impl RunApi {
    pub fn construct_and_start_runtime(config: &String) -> Result<RunApi> {
        let env_config = EnvConfig::get_config();

        let mut command_output = Command::new(env_config.runtime_bin)
            .arg("run")
            .arg("-")
            .stdin(Stdio::piped())
            .spawn()?;

        let mut child_stdin = command_output.stdin.take().expect("Failed to open stdin");

        println!("{}", config);
        child_stdin.write_all(config.as_bytes())?;
        child_stdin.write(b"\n")?;
        child_stdin.flush()?;

        Ok(RunApi {
            runtime_process: command_output,
            killed: false,
        })
    }
}
