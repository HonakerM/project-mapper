use serde::{Deserialize, Serialize};

use crate::config::output::window::WindowConfig;
use crate::config::shared::Component;

// OutputConfig contains
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum OutputConfig {
    Window(WindowConfig),
}

// OutputComponent is the generic component for
// all output types
#[derive(Serialize, Deserialize, Debug)]
pub struct OutputComponentConfig {
    pub uid: u32,
    pub name: String,
    pub config: OutputConfig,
}

// Implmement the Shared component trait to allow name/id fetching
impl Component for OutputComponentConfig {
    fn name(self) -> String {
        self.name.clone()
    }

    fn uid(self) -> u32 {
        self.uid
    }
}
