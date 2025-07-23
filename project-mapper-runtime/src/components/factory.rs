use std::any::type_name_of_val;

use project_mapper_core::runtime_config::{
    effect::{common::EffectConfig, gamma::GammaConfig, EffectComponentConfig},
    input::{test::TestConfig, uri::UriConfig, InputComponentConfig},
    output::{common::OutputConfig, OutputComponentConfig},
    shared::ComponentConfig,
};

use crate::components::{
    effect::{balance::BalanceComponent, gamma::GammaComponent},
    input::{test::TestComponent, uri::UriComponent},
    output::window::WindowComponent,
    shared::Component,
};
use anyhow::{Error, Result, anyhow};

pub fn create_default_component(config: &dyn ComponentConfig) -> Result<Box<dyn Component>> {
    if let Some(output_cfg) = config.as_any().downcast_ref::<OutputComponentConfig>() {
        match &output_cfg.config {
            OutputConfig::Window(_) => {
                let comp = WindowComponent::new(config)?;
                Ok(Box::new(comp))
            }
        }
    } else if let Some(input_cfg) = config.as_any().downcast_ref::<InputComponentConfig>() {
        if let Some(_) = input_cfg.as_any().downcast_ref::<TestConfig>() {
            let comp = TestComponent::new(config)?;
            Ok(Box::new(comp))
        } else if let Some(_) = input_cfg.as_any().downcast_ref::<UriConfig>() {
            let comp = UriComponent::new(config)?;
            Ok(Box::new(comp))
        } else {
            let unknown_name = type_name_of_val(input_cfg.as_any());
            Err(anyhow!("Unknown config type: {}", unknown_name))
        }
    } else if let Some(effect_cfg) = config.as_any().downcast_ref::<EffectComponentConfig>() {
        match &effect_cfg.config {
            EffectConfig::Balance(_) => {
                let comp = BalanceComponent::new(config)?;
                Ok(Box::new(comp))
            }
            EffectConfig::Gamma(_) => {
                let comp = GammaComponent::new(config)?;
                Ok(Box::new(comp))
            }
        }
    } else {
        Err(Error::msg("Unknown component config type"))
    }
}
