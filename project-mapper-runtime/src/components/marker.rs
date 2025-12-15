use std::{
    any::TypeId,
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

use crate::components::shared::Component;
use anyhow::Result;
use project_mapper_core::runtime_config::{
    effect::common::EffectSrcConfigTrait, shared::ComponentConfig,
};

#[derive(Clone)]
pub struct ComponentMarker<CFG, CRT> {
    pub name: &'static str,
    pub type_id: fn() -> TypeId,
    pub config: CFG,
    pub component_creator: CRT,
    /* ... */
}

//pub type DefaultConfig = fn(&mut dyn erased_serde::Deserializer) -> erased_serde::Result<Box<T>>;
pub type DefaultConfig = fn() -> Result<Box<dyn ComponentConfig>>;
pub type ConstructComponent = fn(&dyn ComponentConfig) -> Result<Box<dyn Component>>;

pub type Marker = ComponentMarker<DefaultConfig, ConstructComponent>;

impl Marker {
    pub const fn new(
        name: &'static str,
        type_id: fn() -> TypeId,
        config: DefaultConfig,
        component_creator: ConstructComponent,
    ) -> Self {
        ComponentMarker {
            name,
            type_id,
            config,
            component_creator,
        }
    }
}
