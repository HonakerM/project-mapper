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
        let config = project_mapper_core::loader::load_config(&self.config_path)?;
        let mut app = runtime::Runtime::new(config)?;
        app.run()
    }
}

#[derive(Parser)]
pub struct GetAvailableConfig {}

impl GetAvailableConfig {
    pub fn run(&self) -> Result<()> {
        Ok(())
    }
}

#[derive(Parser)]
pub enum Cli {
    Run(Run),
    GetAvailableConfig(GetAvailableConfig),
}
