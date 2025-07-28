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
use project_mapper_core::runtime_config::{
    input::{InputComponentConfig, test::TestConfig},
    shared::{ComponentConfig, Uid},
};

pub struct TestComponent {
    config: InputComponentConfig,
    element: Element,
    branch: BranchControl,
}

impl Component for TestComponent {
    // runtime lifecycle functions
    // Construct object
    fn new(
        unknown_config: &dyn ComponentConfig,
        pipeline: &gst::Pipeline,
    ) -> Result<TestComponent> {
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
            None => Err(anyhow!("InputComponentConfig is not TestConfig")),
        }?;

        // construct test gstreamer element
        let element = gst::ElementFactory::make("videotestsrc")
            .name(config.name())
            .build()?;

        // Add tee to src element to allow multiple linkages

        let comp = Self {
            branch: BranchControl::new(config.name(), false, true)?,
            config: config,
            element: element,
        };

        // Add elements to the pipelines and sync status
        pipeline.add(&comp.element)?;
        comp.element.sync_state_with_parent()?;

        comp.branch.add_to_pipeline(pipeline)?;
        comp.branch.link_wrapped(&comp.element)?;
        Ok(comp)
    }

    // Run any post init setup functions
    // ! Will probably be removed or edited to have more params
    fn setup(
        &mut self,
        _pipeline: &gst::Pipeline,
        _message_sender: mpsc::Sender<RuntimeMessage>,
        _lookup_func: &dyn ComponentLookupHelper,
    ) -> Result<()> {
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
}
