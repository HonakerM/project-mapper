use std::iter::Map;

use anyhow::Result;
use gst::Element;
use project_mapper_core::runtime_config::shared::{ComponentConfig, Uid};

pub trait ComponentLookupHelper {
    fn lookup(&self, uid: Uid) -> dyn Component;
}

pub trait Component {
    // runtime lifecycle functions
    // Construct object
    fn new(config: &dyn ComponentConfig) -> Result<Self>
    where
        Self: Sized;

    // Run any post init setup functions after all components have been initialized
    // in the pipeline
    fn setup(
        &self,
        pipeline: &gst::Pipeline,
        lookup_func: &dyn ComponentLookupHelper,
    ) -> Result<()>;

    // accessor functions
    fn element(&self) -> &Element;
}

pub trait RunableCompnent {
    // Start this component
    fn start(&self) -> Result<()>;

    // Stop this component
    fn stop(&self) -> Result<()>;
}
