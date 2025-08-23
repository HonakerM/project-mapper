use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex, mpsc},
};

use anyhow::{Error, Result, anyhow};
use gst::Element;
use project_mapper_core::runtime_config::{
    RuntimeConfig,
    shared::{ComponentConfig, Uid},
};

use crate::{components::runtime::DefaultRuntimeComponent, types::message::RuntimeMessage};

// trait to aid in the creation of new components
pub trait ComponentFactory {
    fn create_component(
        &self,
        config: &dyn ComponentConfig,
    ) -> Result<Box<dyn Component>>;
}

pub trait ComponentLookupHelper {
    // factory function to create a component and register it with the helper
    fn new(
        &mut self,
        config: &dyn ComponentConfig,
        factory: &dyn ComponentFactory,
    ) -> Result<()>;
    fn setup(
        &self,
        uid: Uid,
        pipeline: &gst::Pipeline,
        message_sender: mpsc::Sender<RuntimeMessage>,
    ) -> Result<()>;

    fn update(&mut self, config: &dyn ComponentConfig) -> Result<()>;
    // helper function to return a desired component and run setup if it hasn't already
    // start or resume all components. Components must be safe against already initialized
    // state
    fn resume(&self) -> Result<()>;
    // helper to pause all running components
    fn pause(&self) -> Result<()>;
    fn run(
        &self,
        message_broker: Arc<Mutex<mpsc::Receiver<RuntimeMessage>>>,
    ) -> Result<RuntimeMessage>;
    fn destroy_comp(&mut self, uid: &Uid) -> Result<()>;

    // Special function called when exiting the application
    fn destory(&mut self) -> Result<()>;

    // if this helper has a component that requires the main thread to run. ! This must be valid after running
    // setup
    fn has_main_requirement(&self) -> bool;
    fn contains_comp(&self, uid: &Uid) -> bool;
    fn get_comp(&self, uid: &Uid) -> Option<Rc<RefCell<Box<dyn Component>>>>;
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
    ) -> Result<()>;

    // accessor functions
    fn input_element(&self) -> Result<&Element>;
    fn output_element(&self) -> Result<&Element>;
    fn uid(&self) -> Uid;

    /* Runtime Functions */
    // Start this component. Should return quickly
    fn resume(&mut self) -> Result<()> {
        Ok(())
    }
    // pause the component while it's running. This should not delete resources
    fn pause(&mut self) -> Result<()> {
        Ok(())
    }
    // run the component! This should only be called if we have a main requirement
    fn run(
        &self,
        message_receiver: Arc<Mutex<mpsc::Receiver<RuntimeMessage>>>,
    ) -> Result<RuntimeMessage> {
        DefaultRuntimeComponent::manage_events(message_receiver)
    }

    // update a component based on a new config
    fn update_and_link(&mut self, config: &dyn ComponentConfig, 
        lookup_func: &dyn ComponentLookupHelper,
    ) -> Result<()> {
        Ok(())
    }
    // Completely stop and destroy this component
    fn destroy(&mut self) -> Result<()> {
        Ok(())
    }
    // if this component requires running on the main thread.
    // ! Warning: only one component can mark this as true. If multiple
    // components require main then we will raise an error.
    // ! Note: This needs to be correctly set after setup()
    fn requires_main(&self) -> bool {
        false
    }
}
