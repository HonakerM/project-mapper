use std::any::Any;

use serde::{Deserialize, Serialize};

use crate::runtime_config::{
    config::{DEFAULT_ID, DEFAULT_NAME},
    shared::{ComponentConfig, Uid},
};

// Trait representing an output config
// ! I don't know if this is good/okay....
#[typetag::serde(tag = "type")]
pub trait OutputConfigTrait: std::fmt::Debug + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn clone_box(&self) -> Box<dyn OutputConfigTrait>;
}

// Clone support for Box<dyn OutputConfigTrait>
impl Clone for Box<dyn OutputConfigTrait> {
    fn clone(&self) -> Box<dyn OutputConfigTrait> {
        self.clone_box()
    }
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
    pub config: Box<dyn OutputConfigTrait>,

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

    fn dependents(&self) -> Vec<Uid> {
        return vec![self.src_uid];
    }

    fn clone_box(&self) -> Box<dyn ComponentConfig> {
        Box::new(self.clone())
    }
}

impl OutputComponentConfig {
    pub fn default(config: Box<dyn OutputConfigTrait>) -> Self {
        Self {
            uid: DEFAULT_ID,
            src_uid: DEFAULT_ID,
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
