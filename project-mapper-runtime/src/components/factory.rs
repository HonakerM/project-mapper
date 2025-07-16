use project_mapper_core::runtime_config::{
    input::{InputComponentConfig, common::InputConfig},
    output::{OutputComponentConfig, common::OutputConfig},
    shared::ComponentConfig,
};

use crate::components::{
    input::test::TestComponent, output::window::WindowComponent, shared::Component,
};
use anyhow::{Error, Result};

pub fn create_default_component(config: &dyn ComponentConfig) -> Result<Box<dyn Component>> {
    if let Some(output_cfg) = config.as_any().downcast_ref::<OutputComponentConfig>() {
        match &output_cfg.config {
            OutputConfig::Window(_) => {
                let comp = WindowComponent::new(config)?;
                Ok(Box::new(comp))
            }
        }
    } else if let Some(input_cfg) = config.as_any().downcast_ref::<InputComponentConfig>() {
        match &input_cfg.config {
            InputConfig::Test(_) => {
                let comp = TestComponent::new(config)?;
                Ok(Box::new(comp))
            }
            InputConfig::URI(_) => Err(Error::msg("URI Component not yet implemented")),
        }
    } else {
        Err(Error::msg("Unknown component config type"))
    }
}
