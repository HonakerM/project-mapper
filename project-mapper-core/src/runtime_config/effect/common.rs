use std::any::Any;

use serde::{Deserialize, Serialize};

use crate::runtime_config::shared::{ComponentConfig, Uid};

// Trait representing an input config
// ! I don't know if this is good/okay....
#[typetag::serde(tag = "type")]
pub trait EffectConfigTrait: std::fmt::Debug + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn clone_box(&self) -> Box<dyn EffectConfigTrait>;
}
#[typetag::serde(tag = "type")]
pub trait EffectSrcConfigTrait: std::fmt::Debug + Send + Sync {
    fn uid(&self) -> Uid;
    fn as_any(&self) -> &dyn Any;
    fn clone_box(&self) -> Box<dyn EffectSrcConfigTrait>;
}

// Clone support for Box<dyn EffectConfigTrait>
impl Clone for Box<dyn EffectConfigTrait> {
    fn clone(&self) -> Box<dyn EffectConfigTrait> {
        self.clone_box()
    }
}

impl Clone for Box<dyn EffectSrcConfigTrait> {
    fn clone(&self) -> Box<dyn EffectSrcConfigTrait> {
        self.clone_box()
    }
}
// EffectComponent is the generic component for
// all Effect types
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EffectComponentConfig {
    // core component uid
    pub uid: Uid,
    // core component name
    pub name: String,
    // core component config
    pub config: Box<dyn EffectConfigTrait>,

    // what source to use for this Effect
    pub srcs: Vec<Box<dyn EffectSrcConfigTrait>>,
}

// Implmement the Shared component trait to allow name/id fetching
impl ComponentConfig for EffectComponentConfig {
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
        return self.srcs.iter().map(|s| s.uid()).collect();
    }

    fn clone_box(&self) -> Box<dyn ComponentConfig> {
        Box::new(self.clone())
    }
}

// Implement InputConfigTrait for TestConfig
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct DefaultSrcConfig {
    pub uid: Uid,
}

#[typetag::serde(name = "default")]
impl EffectSrcConfigTrait for DefaultSrcConfig {
    fn uid(&self) -> Uid {
        self.uid
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn EffectSrcConfigTrait> {
        Box::new(self.clone())
    }
}
