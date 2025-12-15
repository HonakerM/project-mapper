use std::{
    any::{Any, TypeId, type_name_of_val},
    collections::HashMap,
    default,
};

use project_mapper_core::runtime_config::{
    effect::EffectComponentConfig, output::OutputComponentConfig, shared::ComponentConfig,
};

use crate::components::{
    marker::{ConstructComponent, Marker},
    shared::{Component, ComponentFactory},
};
use anyhow::{Error, Result, anyhow};

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

inventory::collect!(Marker);

pub extern crate inventory;
