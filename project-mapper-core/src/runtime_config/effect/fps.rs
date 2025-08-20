use std::any::Any;

use serde::{Deserialize, Deserializer, Serialize, de};

use crate::runtime_config::{effect::common::EffectConfigTrait, utils::ensure_config_bounds};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct FpsConfig {
    pub max_rate: Option<i32>,
}

// Implement InputConfigTrait for TestConfig
#[typetag::serde]
impl EffectConfigTrait for FpsConfig {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn EffectConfigTrait> {
        Box::new(self.clone())
    }
}
