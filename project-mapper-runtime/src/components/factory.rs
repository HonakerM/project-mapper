use std::{any::type_name_of_val, default};

use project_mapper_core::runtime_config::{
    effect::{balance::BalanceConfig, fps::FpsConfig, gamma::GammaConfig, EffectComponentConfig},
    input::{test::TestConfig, uri::UriConfig, InputComponentConfig},
    output::{window::WindowConfig, OutputComponentConfig},
    shared::ComponentConfig,
};

use crate::components::{
    effect::{balance::BalanceComponent, fps::FpsComponent, gamma::GammaComponent},
    input::{test::TestComponent, uri::UriComponent},
    output::window::WindowComponent,
    shared::{Component, ComponentFactory},
};
use anyhow::{Error, Result, anyhow};

pub fn create_default_component(
    config: &dyn ComponentConfig,
    pipeline: &gst::Pipeline,
) -> Result<Box<dyn Component>> {
    if let Some(output_cfg) = config.as_any().downcast_ref::<OutputComponentConfig>() {
        if let Some(_) = output_cfg.config.as_any().downcast_ref::<WindowConfig>() {
            let comp = WindowComponent::new(config, pipeline)?;
            Ok(Box::new(comp))
        } else {
            let unknown_name = type_name_of_val(&output_cfg.config);
            Err(anyhow!("Unknown config type: {}", unknown_name))
        }
    } else if let Some(input_cfg) = config.as_any().downcast_ref::<InputComponentConfig>() {
        if let Some(_) = input_cfg.config.as_any().downcast_ref::<TestConfig>() {
            let comp = TestComponent::new(config, pipeline)?;
            Ok(Box::new(comp))
        } else if let Some(_) = input_cfg.config.as_any().downcast_ref::<UriConfig>() {
            let comp = UriComponent::new(config, pipeline)?;
            Ok(Box::new(comp))
        } else {
            let unknown_name = type_name_of_val(&input_cfg.config);
            Err(anyhow!("Unknown config type: {}", unknown_name))
        }
    } else if let Some(effect_cfg) = config.as_any().downcast_ref::<EffectComponentConfig>() {
        if let Some(_) = effect_cfg.config.as_any().downcast_ref::<BalanceConfig>() {
            let comp = BalanceComponent::new(config, pipeline)?;
            Ok(Box::new(comp))
        } else if let Some(_) = effect_cfg.config.as_any().downcast_ref::<GammaConfig>() {
            let comp = GammaComponent::new(config, pipeline)?;
            Ok(Box::new(comp))
        } else if let Some(_) = effect_cfg.config.as_any().downcast_ref::<FpsConfig>() {
            let comp = FpsComponent::new(config, pipeline)?;
            Ok(Box::new(comp))
        } else {
            let unknown_name = type_name_of_val(&effect_cfg.config);
            Err(anyhow!("Unknown config type: {}", unknown_name))
        }
    } else {
        Err(Error::msg("Unknown component config type"))
    }
}

// default factory for creating components
#[derive(Default)]
pub struct DefaultComponentFactory {}

impl ComponentFactory for DefaultComponentFactory {
    fn create_component(
        &self,
        config: &dyn ComponentConfig,
        pipeline: &gst::Pipeline,
    ) -> Result<Box<dyn Component>> {
        create_default_component(config, pipeline)
    }
}
