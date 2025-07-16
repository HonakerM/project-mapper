use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use gst::Element;
use project_mapper_core::runtime_config::shared::{ComponentConfig, Uid};

pub trait ComponentLookupHelper {
    fn lookup_and_setup(
        &self,
        uid: Uid,
        pipeline: &gst::Pipeline,
    ) -> Result<Rc<RefCell<Box<dyn Component>>>>;

    fn has_main_requirement(&self) -> bool;
}

pub trait Component {
    // runtime lifecycle functions
    // Construct object
    fn new(config: &dyn ComponentConfig) -> Result<Self>
    where
        Self: Sized;

    // Run any post init setup functions after all components have been initialized
    // in the pipeline. We can garuntee these functions will all be ran in the same thread
    // and one after another. There is no garuntee on the order
    fn setup(
        &mut self,
        pipeline: &gst::Pipeline,
        lookup_func: &dyn ComponentLookupHelper,
    ) -> Result<()>;
    fn has_setup(&self) -> bool {
        return false;
    }

    // accessor functions
    fn element(&self) -> &Element;
    fn uid(&self) -> Uid;

    /* Runtime Functions */
    // Start this component. Should only hold/run if requires_main
    // is true
    fn start_or_run(&mut self, _pipeline: &gst::Pipeline) -> Result<()> {
        Ok(())
    }

    // Completely stop and destroy this component
    fn destroy(&mut self) -> Result<()> {
        Ok(())
    }
    // if this component requires running on the main thread.
    // ! Warning: only one component can mark this as true. If multiple
    // components require main then we will raise an error.
    // ! Note: This needs to be correctly set after new()
    fn requires_main(&self) -> bool {
        false
    }
}
