use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex, mpsc},
};

use anyhow::{Error, Result, anyhow};
use gst::Element;
use project_mapper_core::runtime_config::shared::{ComponentConfig, Uid};

use crate::{components::runtime::DefaultRuntimeComponent, types::message::RuntimeMessage};

pub trait ComponentLookupHelper {
    // factory function to create a component and register it with the helper
    fn create_and_insert_comp(&mut self, config: &dyn ComponentConfig) -> Result<()>;
    // helper function to return a desired component and run setup if it hasn't already
    fn lookup_and_setup(
        &self,
        uid: Uid,
        pipeline: &gst::Pipeline,
        message_sender: mpsc::Sender<RuntimeMessage>,
    ) -> Result<Rc<RefCell<Box<dyn Component>>>>;
    // start or resume all components. Components must be safe against already initialized
    // state
    fn start_or_resume(&self, pipeline: &gst::Pipeline) -> Result<()>;
    fn stop(&self) -> Result<()>;
    fn run(
        &self,
        pipeline: &gst::Pipeline,
        message_broker: Arc<Mutex<mpsc::Receiver<RuntimeMessage>>>,
    ) -> Result<RuntimeMessage>;
    // Special function called when exiting the application
    fn destory(&self) -> Result<()>;

    // if this helper has a component that requires the main thread to run. ! This must be valid after running
    // setup
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
        message_sender: mpsc::Sender<RuntimeMessage>,
        lookup_func: &dyn ComponentLookupHelper,
    ) -> Result<()>;
    fn has_setup(&self) -> bool {
        return false;
    }

    // accessor functions
    fn element(&self) -> Result<&Element>;
    fn uid(&self) -> Uid;

    /* Runtime Functions */
    // Start this component. Should return quickly
    fn start_or_resume(&mut self, _pipeline: &gst::Pipeline) -> Result<()> {
        Ok(())
    }
    //
    fn stop(&mut self) -> Result<()> {
        Ok(())
    }
    fn run(
        &self,
        _pipeline: &gst::Pipeline,
        message_receiver: Arc<Mutex<mpsc::Receiver<RuntimeMessage>>>,
    ) -> Result<RuntimeMessage> {
        DefaultRuntimeComponent::manage_events(message_receiver)
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
