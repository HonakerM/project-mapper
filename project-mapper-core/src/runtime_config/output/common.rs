use serde::{Deserialize, Serialize};

use crate::runtime_config::output::window::WindowConfig;
use crate::runtime_config::shared::{Component, Uid};

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
    // core component uid
    pub uid: Uid,
    // core component name
    pub name: String,
    // core component config
    pub config: OutputConfig,

    // what source to use for this output
    pub src_uid: Uid,
}

// Implmement the Shared component trait to allow name/id fetching
impl Component for OutputComponentConfig {
    fn name(self) -> String {
        self.name.clone()
    }

    fn uid(self) -> Uid {
        self.uid
    }
}
