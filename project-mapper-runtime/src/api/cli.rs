use std::fs;

use clap::Parser;

use crate::runtime;
use anyhow::Result;

#[derive(Parser)]
pub struct Run {
    #[clap(required(true))]
    pub config_path: String,
}

impl Run {
    pub fn run(&self) -> Result<()> {
        println!("attempting to load config from '{}'", self.config_path);
        let config = project_mapper_core::loader::load_config(&self.config_path)?;
        let mut app = runtime::Runtime::new(config)?;
        app.run()
    }
}

#[derive(Parser)]
pub struct GetAvailableConfig {
    #[clap(short, long)]
    pub config_path: Option<String>,
}

impl GetAvailableConfig {
    pub fn run(&self) -> Result<()> {
        let available_config = runtime::options::generate_options()?;
        let config_string = serde_json::to_string(&available_config)?;
        if let Some(path) = &self.config_path {
            Ok(fs::write(path, config_string)?)
        } else {
            println!("{}", config_string);
            Ok(())
        }
    }
}

#[derive(Parser)]
pub enum Cli {
    Run(Run),
    GetAvailableConfig(GetAvailableConfig),
}
