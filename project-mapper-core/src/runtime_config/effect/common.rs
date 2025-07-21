use std::any::Any;

use serde::{Deserialize, Serialize};

use crate::runtime_config::{
    effect::{balance::BalanceConfig, gamma::GammaConfig},
    shared::{ComponentConfig, Uid},
};

// EffectConfig contains
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum EffectConfig {
    Balance(BalanceConfig),
    Gamma(GammaConfig),
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
    pub config: EffectConfig,

    // what source to use for this Effect
    pub src_uid: Uid,
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
}
