use std::sync::mpsc;

use crate::{
    components::{
        branch::BranchControl,
        shared::{Component, ComponentLookupHelper},
    },
    types::message::RuntimeMessage,
};
use anyhow::{anyhow, Error, Result};
use gst::{Element, prelude::*};
use project_mapper_core::runtime_config::{
    input::{test::TestConfig, InputComponentConfig},
    shared::{ComponentConfig, Uid},
};

pub struct TestComponent {
    config: InputComponentConfig,
    element: Element,
    branch: BranchControl,
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

        // ensure we have a test config
        match config.config.as_any().downcast_ref::<TestConfig>() {
            Some(b) => Ok(b.clone()),
            None => Err(anyhow!("InputComponentConfig is not TestConfig"))
        }?;

        // construct test gstreamer element
        let element = gst::ElementFactory::make("videotestsrc")
            .name(config.name())
            .build()?;

        // Add tee to src element to allow multiple linkages

        Ok(Self {
            branch: BranchControl::new(config.name(), false, true)?,
            config: config,
            element: element,
            has_setup: false,
        })
    }

    // Run any post init setup functions
    // ! Will probably be removed or edited to have more params
    fn setup(
        &mut self,
        pipeline: &gst::Pipeline,
        _message_sender: mpsc::Sender<RuntimeMessage>,
        _lookup_func: &dyn ComponentLookupHelper,
    ) -> Result<()> {
        self.has_setup = true;

        // Add elements to the pipelines and sync status
        pipeline.add(&self.element)?;
        self.element.sync_state_with_parent()?;

        self.branch.add_to_pipeline(pipeline)?;
        self.branch.link_wrapped(&self.element)?;

        Ok(())
    }

    // accessor functions
    fn element(&self) -> Result<&Element> {
        // return the tee element since that's what people should
        // be linking against
        self.branch.get_output()
    }
    fn uid(&self) -> Uid {
        return self.config.uid();
    }
    fn has_setup(&self) -> bool {
        return self.has_setup;
    }
}
