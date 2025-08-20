use std::any::Any;

use serde::{Deserialize, Serialize};

use crate::runtime_config::input::common::InputConfigTrait;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]

pub struct TestConfig {
    pub fps: i32,
}

impl Default for TestConfig {
    fn default() -> Self {
        TestConfig { fps: 30 }
    }
}

// Implement InputConfigTrait for TestConfig
#[typetag::serde]
impl InputConfigTrait for TestConfig {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn InputConfigTrait> {
        Box::new(self.clone())
    }
}
