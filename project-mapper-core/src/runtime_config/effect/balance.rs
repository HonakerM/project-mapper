use std::any::Any;

use serde::{Deserialize, Deserializer, Serialize, de};

use crate::runtime_config::{effect::common::EffectConfigTrait, utils::validation::ensure_config_bounds};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct BalanceConfig {
    #[serde(deserialize_with = "deserialize_bounded_brightness")]
    pub brightness: Option<f64>,
    #[serde(deserialize_with = "deserialize_bounded_contrast")]
    pub contrast: Option<f64>,
    #[serde(deserialize_with = "deserialize_bounded_hue")]
    pub hue: Option<f64>,
    #[serde(deserialize_with = "deserialize_bounded_saturation")]
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

fn deserialize_bounded_brightness<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let some_val = Option::<f64>::deserialize(deserializer)?;
    ensure_config_bounds(some_val, -1.0, 1.0).map_err(|e| {
        de::Error::custom(format!(
            "brightness value {:?} has error: {:#}",
            some_val, e,
        ))
    })
}

fn deserialize_bounded_contrast<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let some_val = Option::<f64>::deserialize(deserializer)?;
    ensure_config_bounds(some_val, 0.0, 2.0).map_err(|e| {
        de::Error::custom(format!("contrast value {:?} has error: {:#}", some_val, e,))
    })
}

fn deserialize_bounded_hue<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let some_val = Option::<f64>::deserialize(deserializer)?;
    ensure_config_bounds(some_val, -1.0, 1.0)
        .map_err(|e| de::Error::custom(format!("hue value {:?} has error: {:#}", some_val, e,)))
}

fn deserialize_bounded_saturation<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let some_val = Option::<f64>::deserialize(deserializer)?;
    ensure_config_bounds(some_val, 0.0, 2.0).map_err(|e| {
        de::Error::custom(format!(
            "saturation value {:?} has error: {:#}",
            some_val, e,
        ))
    })
}
