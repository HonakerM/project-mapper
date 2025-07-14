use serde::{Deserialize, Serialize};

use crate::runtime_config::{input::InputComponentConfig, output::OutputComponentConfig};

// Top-Level Config object for the runtime
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub inputs: Vec<InputComponentConfig>,
    pub outputs: Vec<OutputComponentConfig>,
}
