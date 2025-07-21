use project_mapper_core::runtime_config::{
    effect::{EffectComponentConfig, common::EffectConfig},
    input::{InputComponentConfig, common::InputConfig},
    output::{OutputComponentConfig, common::OutputConfig},
    shared::ComponentConfig,
};

use crate::components::{
    effect::balance::BalanceComponent,
    input::{test::TestComponent, uri::UriComponent},
    output::window::WindowComponent,
    shared::Component,
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
            InputConfig::URI(_) => {
                let comp = UriComponent::new(config)?;
                Ok(Box::new(comp))
            }
        }
    } else if let Some(effect_cfg) = config.as_any().downcast_ref::<EffectComponentConfig>() {
        match &effect_cfg.config {
            EffectConfig::Balance(_) => {
                let comp = BalanceComponent::new(config)?;
                Ok(Box::new(comp))
            }
        }
    } else {
        Err(Error::msg("Unknown component config type"))
    }
}
