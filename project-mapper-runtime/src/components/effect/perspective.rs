use std::sync::mpsc;

use crate::{
    components::{
        branch::BranchControl,
        shared::{Component, ComponentLookupHelper},
    },
    types::message::RuntimeMessage,
};
use anyhow::{Error, Result, anyhow};
use gst::{Element, prelude::*};
use log::debug;
use project_mapper_core::runtime_config::{
    effect::{EffectComponentConfig, perspective::PerspectiveConfig},
    shared::{ComponentConfig, Uid},
};

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
    fn input_element(&self) -> Result<&Element> {
        self.branch.get_input()
    }
    fn output_element(&self) -> Result<&Element> {
        self.branch.get_output()
    }
    fn uid(&self) -> Uid {
        return self.config.uid();
    }
}
