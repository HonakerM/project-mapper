use anyhow::Result;
use gst::Element;
use project_mapper_core::runtime_config::shared::ComponentConfig;

pub trait Component {
    // runtime lifecycle functions
    // Construct object
    fn new(config: &dyn ComponentConfig) -> Result<Self>
    where
        Self: Sized;
    // Run any post init setup functions
    // ! Will probably be removed or edited to have more params
    fn setup(&self) -> Result<()>;
    // Run things required by the component
    fn run(&self) -> Result<()>;

    // function used to link this component to other components.
    // for now this should always flow src to sinks. E.g. this should
    // never be called on final output components
    fn link_to(element: &Element, pipeline: &gst::Pipeline) -> Result<()>;

    // function to check if this component's run requires a thread
    fn requires_thread(&self) -> bool {
        false
    }

    // accessor functions
    fn element(&self) -> &Element;
}
