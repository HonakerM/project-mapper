use std::any::Any;

use const_default::ConstDefault;
use serde::{Deserialize, Serialize};
use typetag;

use crate::runtime_config::{
    config::{DEFAULT_ID, DEFAULT_NAME},
    shared::{ComponentConfig, Uid},
};

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

impl InputComponentConfig {
    pub fn default(config: Box<dyn InputConfigTrait>) -> Self {
        Self {
            uid: DEFAULT_ID,
            name: DEFAULT_NAME.to_owned(),
            config,
        }
    }
    pub fn default_name() -> String {
        DEFAULT_NAME.to_owned()
    }
    pub fn default_id() -> Uid {
        DEFAULT_ID
    }
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

    fn dependents(&self) -> Vec<Uid> {
        return vec![];
    }

    fn clone_box(&self) -> Box<dyn ComponentConfig> {
        Box::new(self.clone())
    }
}
