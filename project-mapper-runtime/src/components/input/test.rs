use crate::components::shared::{Component, ComponentLookupHelper};
use anyhow::{Error, Result};
use gst::{Element, prelude::*};
use project_mapper_core::runtime_config::{
    input::{InputComponentConfig, common::InputConfig},
    shared::{ComponentConfig, Uid},
};

pub struct TestComponent {
    config: InputComponentConfig,
    element: Element,
    tee_element: Element,
    has_setup: bool,
}

impl Component for TestComponent {
    // runtime lifecycle functions
    // Construct object
    fn new(unknown_config: &dyn ComponentConfig) -> Result<TestComponent> {
        // parse config and ensure it's correct types
        let config: InputComponentConfig = match unknown_config
            .as_any()
            .downcast_ref::<InputComponentConfig>()
        {
            Some(b) => Ok(b.clone()),
            None => Err(Error::msg(
                "ComponentConfig can not be typed to InputComponentConfig",
            )),
        }?;
        if !matches!(config.config, InputConfig::Test(_)) {
            return Err(Error::msg("Component Config is not proper type"));
        }

        // construct test gstreamer element
        let element = gst::ElementFactory::make("videotestsrc")
            .name(config.name())
            .build()?;

        // Add tee to src element to allow multiple linkages
        let tee_name: String = format!("tee-{}", config.name());
        let src_tee = gst::ElementFactory::make("tee").name(tee_name).build()?;

        Ok(Self {
            config: config,
            element: element,
            tee_element: src_tee,
            has_setup: false,
        })
    }

    // Run any post init setup functions
    // ! Will probably be removed or edited to have more params
    fn setup(
        &mut self,
        pipeline: &gst::Pipeline,
        _lookup_func: &dyn ComponentLookupHelper,
    ) -> Result<()> {
        self.has_setup = true;

        // Add elements to the pipelines and sync status
        pipeline.add(&self.element)?;
        pipeline.add(&self.tee_element)?;

        self.tee_element.sync_state_with_parent()?;
        self.element.sync_state_with_parent()?;

        self.element.link(&self.tee_element)?;

        Ok(())
    }

    // accessor functions
    fn element(&self) -> &Element {
        // return the tee element since that's what people should
        // be linking against
        &self.tee_element
    }
    fn uid(&self) -> Uid {
        return self.config.uid();
    }
    fn has_setup(&self) -> bool {
        return self.has_setup;
    }
}
