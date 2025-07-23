use std::any::Any;

use serde::{Deserialize, Serialize};

use crate::runtime_config::effect::common::EffectConfigTrait;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct GammaConfig {
    pub gamma: Option<f64>,
}

// Implement InputConfigTrait for TestConfig
#[typetag::serde]
impl EffectConfigTrait for GammaConfig {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn EffectConfigTrait> {
        Box::new(self.clone())
    }
}
