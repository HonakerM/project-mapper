use std::any::Any;

use serde::{Deserialize, Serialize};

use crate::runtime_config::input::common::InputConfigTrait;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UriConfig {
    pub uri: String,
}

// Implement InputConfigTrait for TestConfig
#[typetag::serde]
impl InputConfigTrait for UriConfig {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn InputConfigTrait> {
        Box::new(self.clone())
    }
}