use std::any::Any;

use serde::{Deserialize, Serialize};

use crate::runtime_config::output::window::WindowConfig;
use crate::runtime_config::shared::{ComponentConfig, Uid};

// OutputConfig contains
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum OutputConfig {
    Window(WindowConfig),
}

// OutputComponent is the generic component for
// all output types
#[derive(Serialize, Deserialize, Debug, Clone)]
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
impl ComponentConfig for OutputComponentConfig {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn uid(&self) -> Uid {
        self.uid
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
