use std::any::Any;

use serde::{Deserialize, Deserializer, Serialize, de};

use crate::runtime_config::{effect::common::EffectConfigTrait, utils::ensure_config_bounds};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct GammaConfig {
    #[serde(deserialize_with = "deserialize_bounded_gamma")]
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

fn deserialize_bounded_gamma<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let some_val = Option::<f64>::deserialize(deserializer)?;
    ensure_config_bounds(some_val, 0.01, 10.0)
        .map_err(|e| de::Error::custom(format!("gamma value {:?} has error: {:#}", some_val, e,)))
}
