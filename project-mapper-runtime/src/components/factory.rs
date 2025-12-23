use std::{
    any::{Any, TypeId, type_name_of_val},
    collections::HashMap,
    default,
};

use project_mapper_core::runtime_config::{
    effect::EffectComponentConfig, output::OutputComponentConfig, shared::ComponentConfig,
};

use crate::components::{
    marker::{ConstructComponent, Marker, type_id_of},
    shared::{Component, ComponentFactory},
};
use anyhow::{Error, Result, anyhow};
use log::trace;

// default factory for creating components
pub struct DefaultComponentFactory {
    input_components: Vec<ConstructComponent>,
    effect_components: Vec<ConstructComponent>,
    output_components: Vec<ConstructComponent>,
}

impl Default for DefaultComponentFactory {
    fn default() -> Self {
        let mut input_components = Vec::new();
        let mut effect_components = Vec::new();
        let mut output_components = Vec::new();

        println!("Looking For Components");
        for marker in inventory::iter::<Marker> {
            let boxed_config = (marker.config)().unwrap();

            println!(
                "Found Component Marker: {} for Config: {}",
                marker.name,
                type_name_of_val(boxed_config.as_ref())
            );

            if boxed_config
                .as_any()
                .downcast_ref::<project_mapper_core::runtime_config::input::InputComponentConfig>()
                .is_some()
            {
                input_components.push(marker.component_creator.clone());
            } else if boxed_config
                .as_any()
                .downcast_ref::<project_mapper_core::runtime_config::effect::EffectComponentConfig>(
                )
                .is_some()
            {
                effect_components.push(marker.component_creator.clone());
            } else if boxed_config
                .as_any()
                .downcast_ref::<project_mapper_core::runtime_config::output::OutputComponentConfig>(
                )
                .is_some()
            {
                output_components.push(marker.component_creator.clone());
            } else {
                panic!("Unknown component config type");
            }
        }

        DefaultComponentFactory {
            input_components,
            effect_components,
            output_components,
        }
    }
}

impl ComponentFactory for DefaultComponentFactory {
    fn create_component(&self, config: &dyn ComponentConfig) -> Result<Box<dyn Component>> {
        println!("Trying to find component: {:?}", config);

        let vec_to_iter = if config
            .as_any()
            .downcast_ref::<project_mapper_core::runtime_config::input::InputComponentConfig>()
            .is_some()
        {
            &self.input_components
        } else if config
            .as_any()
            .downcast_ref::<project_mapper_core::runtime_config::effect::EffectComponentConfig>()
            .is_some()
        {
            &self.effect_components
        } else if config
            .as_any()
            .downcast_ref::<project_mapper_core::runtime_config::output::OutputComponentConfig>()
            .is_some()
        {
            &self.output_components
        } else {
            panic!("Unknown component config type");
        };

        for (constructor) in vec_to_iter.iter() {
            match constructor(config) {
                Ok(comp) => {
                    return Ok(comp);
                }
                Err(err) => {
                    trace!("Constructor did not match for component: {:?}", err);
                }
            }
        }
        return Err(Error::msg("No matching component constructor found"));
    }
}

inventory::collect!(Marker);

pub extern crate inventory;
