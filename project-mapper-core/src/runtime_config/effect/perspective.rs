use std::any::Any;

use serde::{Deserialize, Serialize};

use crate::runtime_config::effect::common::EffectConfigTrait;

/// Configuration for the Perspective effect component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerspectiveConfig {
    /// 3x3 transformation matrix in row-major order.
    pub matrix: [f64; 9],
}

#[typetag::serde]
impl EffectConfigTrait for PerspectiveConfig {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn EffectConfigTrait> {
        Box::new(self.clone())
    }
}
