use serde::{Deserialize, Serialize};

use crate::runtime_config::output::OutputComponentConfig;

// Top-Level Config object for the runtime
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub outputs: Vec<OutputComponentConfig>,
}
