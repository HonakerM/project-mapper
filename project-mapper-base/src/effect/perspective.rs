use std::sync::mpsc;

use anyhow::{Error, Result, anyhow};
use log::debug;
use project_mapper_core::{
    runtime_config::{
        effect::EffectComponentConfig,
        shared::{ComponentConfig, Uid},
    },
    types::openapi::OpenAPISchema,
};
use project_mapper_runtime::gst::{Element, prelude::*};
use project_mapper_runtime::{
    components::{
        branch::BranchControl,
        shared::{Component, ComponentLookupHelper},
    },
    gst,
    types::message::RuntimeMessage,
};
use schemars::{JsonSchema, schema_for};

use std::any::Any;

use serde::{Deserialize, Serialize};

use project_mapper_core::runtime_config::effect::common::EffectConfigTrait;

/// Configuration for the Perspective effect component.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PerspectiveConfig {
    /// 3x3 transformation matrix in row-major order.
    pub matrix: [f64; 9],
}

impl PerspectiveConfig {
    pub fn openapi_schema() -> OpenAPISchema {
        let mut schema_val = serde_json::to_value(schema_for!(PerspectiveConfig)).unwrap();
        match schema_val.as_object_mut() {
            Some(map) => {
                map.insert(
                    "description".to_string(),
                    serde_json::Value::String("Requires src type `default`".to_string()),
                );
            }
            None => {}
        };
        schema_val.try_into().unwrap()
    }
}
impl Default for PerspectiveConfig {
    fn default() -> Self {
        Self {
            matrix: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        }
    }
}

#[typetag::serde]
impl EffectConfigTrait for PerspectiveConfig {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn EffectConfigTrait> {
        Box::new(self.clone())
    }
}

pub struct PerspectiveComponent {
    config: EffectComponentConfig,
    element: Element,
    branch: BranchControl,
}

impl PerspectiveComponent {
    fn update_config(element: &gst::Element, config: PerspectiveConfig) -> Result<()> {
        debug!("Updating perspective component with config: {:?}", config);
        let g_array = gst::glib::ValueArray::new(config.matrix);
        element.set_property("matrix", g_array);
        Ok(())
    }
}

#[project_mapper_macro::effect_component(config = {PerspectiveConfig::default()}, schema = {PerspectiveConfig::openapi_schema().to_json_value()})]
impl Component for PerspectiveComponent {
    // runtime lifecycle functions
    // Construct object
    fn new(unknown_config: &dyn ComponentConfig) -> Result<PerspectiveComponent> {
        // parse config and ensure it's correct types
        let config: EffectComponentConfig = match unknown_config
            .as_any()
            .downcast_ref::<EffectComponentConfig>()
        {
            Some(b) => Ok(b.clone()),
            None => Err(Error::msg(
                "ComponentConfig can not be typed to EffectComponentConfig",
            )),
        }?;

        // construct element
        let perspective_config = match config.config.as_any().downcast_ref::<PerspectiveConfig>() {
            Some(b) => Ok(b.clone()),
            None => Err(anyhow!(
                "PerspectiveComponentConfig is not PerspectiveConfig"
            )),
        }?;
        let element = gst::ElementFactory::make("perspective")
            .name(config.name())
            .build()?;
        PerspectiveComponent::update_config(&element, perspective_config)?;

        let branch = BranchControl::new(config.name(), true, true)?;
        let comp = Self {
            config: config,
            element: element,
            branch: branch,
        };

        Ok(comp)
    }

    // Run any post init setup functions
    fn setup(
        &mut self,
        pipeline: &gst::Pipeline,
        message_sender: mpsc::Sender<RuntimeMessage>,
    ) -> Result<()> {
        // config the elements in the pipeline
        pipeline.add(&self.element)?;
        self.element.sync_state_with_parent()?;

        // ensure the branch is correctly setup and wrap the parent element
        self.branch.add_to_pipeline(pipeline)?;
        self.branch.link_wrapped(&self.element)?;

        Ok(())
    }

    fn update(&mut self, config: &dyn ComponentConfig) -> Result<()> {
        // parse config and ensure it's correct types
        let config: EffectComponentConfig =
            match config.as_any().downcast_ref::<EffectComponentConfig>() {
                Some(b) => Ok(b.clone()),
                None => Err(Error::msg(
                    "ComponentConfig can not be typed to EffectComponentConfig",
                )),
            }?;

        // update config
        let perspective_config = match config.config.as_any().downcast_ref::<PerspectiveConfig>() {
            Some(b) => Ok(b.clone()),
            None => Err(anyhow!(
                "PerspectiveComponentConfig is not PerspectiveConfig"
            )),
        }?;
        PerspectiveComponent::update_config(&self.element, perspective_config)?;

        if self.config.srcs.len() != 1 {
            return Err(anyhow!(
                "Perspective component must have exactly one source"
            ));
        }

        Ok(())
    }

    // accessor functions
    fn input_element(&self) -> Option<&Element> {
        self.branch.get_input()
    }
    fn output_element(&self) -> Option<&Element> {
        self.branch.get_output()
    }
    fn uid(&self) -> Uid {
        return self.config.uid();
    }
}
