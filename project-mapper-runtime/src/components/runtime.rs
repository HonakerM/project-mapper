use std::any::Any;

use crate::components::shared::{Component, ComponentLookupHelper};
use anyhow::{Result, anyhow};
use project_mapper_core::runtime_config::shared::{ComponentConfig, Uid};

pub static DEFAULT_ID: Uid = -1;

// The default runtime component is used when there is no other
// component that requires main
pub struct DefaultRuntimeComponent {}

impl DefaultRuntimeComponent {
    pub fn new_config() -> Result<DefaultRuntimeComponent> {
        Ok(DefaultRuntimeComponent {})
    }
}

impl Component for DefaultRuntimeComponent {
    fn new(_config: &dyn ComponentConfig) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {})
    }

    fn setup(
        &mut self,
        _pipeline: &gst::Pipeline,
        _lookup_func: &dyn ComponentLookupHelper,
    ) -> Result<()> {
        Ok(())
    }

    fn uid(&self) -> Uid {
        DEFAULT_ID
    }
    fn element(&self) -> Result<&gst::Element> {
        Err(anyhow!("Runtime Component does not have elements"))
    }
    fn requires_main(&self) -> bool {
        true
    }
}

impl ComponentConfig for DefaultRuntimeComponent {
    fn name(&self) -> String {
        "DefaultRuntimeComponent".to_string()
    }
    fn uid(&self) -> Uid {
        DEFAULT_ID
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
