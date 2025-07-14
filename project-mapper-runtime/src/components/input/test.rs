use crate::components::shares::Component;
use anyhow::{Error, Result};
use gst::Element;
use project_mapper_core::runtime_config::{
    input::{InputComponentConfig, common::InputConfig},
    shared::ComponentConfig,
};

struct TestComponent {
    config: InputComponentConfig,
    element: Element,
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

        Ok(Self {
            config: config,
            element: element,
        })
    }
    // Run any post init setup functions
    // ! Will probably be removed or edited to have more params
    fn setup(&self) -> Result<()> {
        Ok(())
    }
    // Run things required by the component
    fn run(&self) -> Result<()> {
        Ok(())
    }

    // function used to link this component to other components.
    // for now this should always flow src to sinks. E.g. this should
    // never be called on final output components
    fn link_to(element: &Element, pipeline: &gst::Pipeline) -> Result<()> {
        Ok(())
    }

    // function to check if this component's run requires a thread
    fn requires_thread(&self) -> bool {
        false
    }

    // accessor functions
    fn element(&self) -> &Element {
        &self.element
    }
}
