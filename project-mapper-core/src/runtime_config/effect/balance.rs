use std::any::Any;

use serde::{Deserialize, Serialize};

use crate::runtime_config::effect::common::EffectConfigTrait;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct BalanceConfig {
    pub brightness: Option<f64>,
    pub contrast: Option<f64>,
    pub hue: Option<f64>,
    pub saturation: Option<f64>,
}

// Implement InputConfigTrait for TestConfig
#[typetag::serde]
impl EffectConfigTrait for BalanceConfig {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn EffectConfigTrait> {
        Box::new(self.clone())
    }
}
