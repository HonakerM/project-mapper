use clap::Parser;

#[derive(Parser)]
pub struct Run {
    #[clap(required(true))]
    pub config_path: String,
}

#[derive(Parser)]
pub struct GetAvailableConfig {}

#[derive(Parser)]
pub enum Cli {
    Run(Run),
    GetAvailableConfig(GetAvailableConfig),
}
