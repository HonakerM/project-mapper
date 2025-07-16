use std::{cell::RefCell, iter::Map, rc::Rc};

use anyhow::Result;
use gst::Element;
use project_mapper_core::runtime_config::shared::{ComponentConfig, Uid};

pub trait ComponentLookupHelper {
    fn lookup_and_setup(
        &self,
        uid: Uid,
        pipeline: &gst::Pipeline,
    ) -> Result<Rc<RefCell<Box<dyn Component>>>>;
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

    // accessor functions
    fn element(&self) -> &Element;
    fn uid(&self) -> Uid;

    // Start this component.
    fn start(&mut self, pipeline: &gst::Pipeline) -> Result<()> {
        Ok(())
    }

    // Completely stop and destroy this component
    fn destroy(&mut self) -> Result<()> {
        Ok(())
    }
}
