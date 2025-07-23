use std::any::Any;

use serde::{Deserialize, Serialize};
use typetag;

use crate::runtime_config::shared::{ComponentConfig, Uid};

// Trait representing an input config
// ! I don't know if this is good/okay....
#[typetag::serde(tag = "type")]
pub trait InputConfigTrait: std::fmt::Debug + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn clone_box(&self) -> Box<dyn InputConfigTrait>;
}

// Clone support for Box<dyn InputConfigTrait>
impl Clone for Box<dyn InputConfigTrait> {
    fn clone(&self) -> Box<dyn InputConfigTrait> {
        self.clone_box()
    }
}

// InputComponentConfig is now driven by trait object
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InputComponentConfig {
    pub uid: Uid,
    pub name: String,
    pub config: Box<dyn InputConfigTrait>,
}

// Implement the shared component trait
impl ComponentConfig for InputComponentConfig {
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
