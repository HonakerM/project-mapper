use std::{
    any::{Any, TypeId, type_name_of_val},
    collections::HashMap,
    default,
};

use project_mapper_core::runtime_config::{
    effect::{
        EffectComponentConfig, balance::BalanceConfig, fps::FpsConfig, gamma::GammaConfig,
        perspective::PerspectiveConfig,
    },
    input::{InputComponentConfig, test::TestConfig, uri::UriConfig},
    output::{OutputComponentConfig, window::WindowConfig},
    shared::ComponentConfig,
};

use crate::components::{
    effect::{
        balance::BalanceComponent, fps::FpsComponent, gamma::GammaComponent,
        perspective::PerspectiveComponent,
    },
    input::uri::UriComponent,
    marker::{ConstructComponent, Marker},
    output::window::WindowComponent,
    shared::{Component, ComponentFactory},
};
use anyhow::{Error, Result, anyhow};

pub fn create_default_component(config: &dyn ComponentConfig) -> Result<Box<dyn Component>> {
    if let Some(output_cfg) = config.as_any().downcast_ref::<OutputComponentConfig>() {
        if let Some(_) = output_cfg.config.as_any().downcast_ref::<WindowConfig>() {
            let comp = WindowComponent::new(config)?;
            Ok(Box::new(comp))
        } else {
            let unknown_name = type_name_of_val(&output_cfg.config);
            Err(anyhow!("Unknown config type: {}", unknown_name))
        }
    } else if let Some(input_cfg) = config.as_any().downcast_ref::<InputComponentConfig>() {
        if let Some(_) = input_cfg.config.as_any().downcast_ref::<UriConfig>() {
            let comp = UriComponent::new(config)?;
            Ok(Box::new(comp))
        } else {
            let unknown_name = type_name_of_val(&input_cfg.config);
            Err(anyhow!("Unknown config type: {}", unknown_name))
        }
    } else if let Some(effect_cfg) = config.as_any().downcast_ref::<EffectComponentConfig>() {
        if let Some(_) = effect_cfg.config.as_any().downcast_ref::<BalanceConfig>() {
            let comp = BalanceComponent::new(config)?;
            Ok(Box::new(comp))
        } else if let Some(_) = effect_cfg.config.as_any().downcast_ref::<GammaConfig>() {
            let comp = GammaComponent::new(config)?;
            Ok(Box::new(comp))
        } else if let Some(_) = effect_cfg.config.as_any().downcast_ref::<FpsConfig>() {
            let comp = FpsComponent::new(config)?;
            Ok(Box::new(comp))
        } else if let Some(_) = effect_cfg
            .config
            .as_any()
            .downcast_ref::<PerspectiveConfig>()
        {
            let comp = PerspectiveComponent::new(config)?;
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
pub struct DefaultComponentFactory {
    lookup_factory: Vec<ConstructComponent>,
}

impl Default for DefaultComponentFactory {
    fn default() -> Self {
        let mut factory = vec![];

        println!("Looking For Components");
        for marker in inventory::iter::<Marker> {
            factory.push(marker.component_creator.clone());
        }

        DefaultComponentFactory {
            lookup_factory: factory,
        }
    }
}

impl ComponentFactory for DefaultComponentFactory {
    fn create_component(&self, config: &dyn ComponentConfig) -> Result<Box<dyn Component>> {
        println!("Trying to find component: {:?}", config);
        if let Ok(comp) = create_default_component(config) {
            return Ok(comp);
        } else {
            for constructor in &self.lookup_factory {
                match constructor(config) {
                    Ok(comp) => {
                        return Ok(comp);
                    }
                    Err(err) => {
                        println!("Constructor did not match for component: {:?}", err);
                    }
                }
            }
            return Err(Error::msg("No matching component constructor found"));
        }
    }
}

inventory::collect!(Marker);

pub extern crate inventory;
