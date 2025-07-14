use serde::{Deserialize, Serialize};

use crate::runtime_config::{
    input::{test::TestConfig, uri::UriConfig},
    shared::{Component, Uid},
};

// InputConfig contains
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum InputConfig {
    Test(TestConfig),
    URI(UriConfig),
}

// InputComponent is the generic component for
// all Input types
#[derive(Serialize, Deserialize, Debug)]
pub struct InputComponentConfig {
    // core component uid
    pub uid: Uid,
    // core component name
    pub name: String,
    // core component config
    pub config: InputConfig,
}

// Implmement the Shared component trait to allow name/id fetching
impl Component for InputComponentConfig {
    fn name(self) -> String {
        self.name.clone()
    }

    fn uid(self) -> Uid {
        self.uid
    }
}
