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
    input::{test::TestComponent, uri::UriComponent},
    marker::{ConstructComponent, Marker},
    output::window::WindowComponent,
    shared::{Component, ComponentFactory},
};
use anyhow::{Error, Result, anyhow};

// default factory for creating components
pub struct DefaultComponentFactory {
    lookup_factory: HashMap<TypeId, ConstructComponent>,
}

impl Default for DefaultComponentFactory {
    fn default() -> Self {
        let mut factory = HashMap::new();

        for marker in inventory::iter::<Marker> {
            factory.insert(
                (marker.config)().type_id(),
                marker.component_creator.clone(),
            );
        }

        DefaultComponentFactory {
            lookup_factory: factory,
        }
    }
}

impl ComponentFactory for DefaultComponentFactory {
    fn create_component(&self, config: &dyn ComponentConfig) -> Result<Box<dyn Component>> {
        let type_id = config.as_any().type_id();
        if let Some(creator) = self.lookup_factory.get(&type_id) {
            creator(config)
        } else {
            Err(Error::msg("No component creator found for given config"))
        }
    }
}

inventory::collect!(Marker);

pub extern crate inventory;
