use std::{
    any::{TypeId, type_name},
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

use crate::components::shared::Component;
use anyhow::Result;
use project_mapper_core::{
    available_config::config::AvailableConfigType,
    runtime_config::{effect::common::EffectSrcConfigTrait, shared::ComponentConfig},
};

#[derive(Clone)]
pub struct ComponentMarker<CFG, CRT, ACG> {
    pub name: &'static str,
    pub type_id: fn() -> TypeId,
    pub config: CFG,
    pub component_creator: CRT,
    pub available_config: ACG,
}

//pub type DefaultConfig = fn(&mut dyn erased_serde::Deserializer) -> erased_serde::Result<Box<T>>;
pub type DefaultConfig = fn() -> Result<Box<dyn ComponentConfig>>;
pub type ConstructComponent = fn(&dyn ComponentConfig) -> Result<Box<dyn Component>>;
pub type AvailablaeConfig = fn() -> Result<Box<AvailableConfigType>>;

pub type Marker = ComponentMarker<DefaultConfig, ConstructComponent, AvailablaeConfig>;

impl Marker {
    pub const fn new(
        name: &'static str,
        type_id: fn() -> TypeId,
        config: DefaultConfig,
        component_creator: ConstructComponent,
        available_config: AvailablaeConfig,
    ) -> Self {
        ComponentMarker {
            name,
            type_id,
            available_config,
            config,
            component_creator,
        }
    }
}

pub fn type_id_of<T: 'static>(_val: T) -> TypeId {
    TypeId::of::<T>()
}

pub fn name_of<T: 'static>(_val: T) -> String {
    type_name::<T>().to_string()
}
