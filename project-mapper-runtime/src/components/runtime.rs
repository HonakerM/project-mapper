use std::{
    any::Any,
    sync::{Arc, Mutex, mpsc},
};

use crate::{
    components::shared::{Component, ComponentLookupHelper},
    types::message::RuntimeMessage,
};
use anyhow::{Result, anyhow};
use project_mapper_core::runtime_config::{config::{DEFAULT_ID, DEFAULT_NAME}, shared::{ComponentConfig, Uid}};


// The default runtime component is used when there is no other
// component that requires main
#[derive(Debug, Clone)]
pub struct DefaultRuntimeComponent {}

impl DefaultRuntimeComponent {
    pub fn get_default_uid() -> Uid {
        DEFAULT_ID
    }
    pub fn new_config() -> Result<DefaultRuntimeComponent> {
        Ok(DefaultRuntimeComponent {})
    }

    pub fn manage_events(
        message_receiver: Arc<Mutex<mpsc::Receiver<RuntimeMessage>>>,
    ) -> Result<RuntimeMessage> {
        let recv = message_receiver
            .lock()
            .map_err(|_| anyhow!("Unable to lock message receiver due to thread panic"))?;
        Ok(recv.recv()?)
    }
}

impl Component for DefaultRuntimeComponent {
    fn new(_config: &dyn ComponentConfig) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {})
    }

    fn setup(
        &mut self,
        _pipeline: &gst::Pipeline,
        _message_sender: mpsc::Sender<RuntimeMessage>,
    ) -> Result<()> {
        Ok(())
    }

    fn uid(&self) -> Uid {
        DEFAULT_ID
    }
    fn input_element(&self) -> Option<&gst::Element> {
        None
    }
    fn output_element(&self) -> Option<&gst::Element> {
        None
    }
    fn requires_main(&mut self) -> bool {
        true
    }
}

impl ComponentConfig for DefaultRuntimeComponent {
    fn name(&self) -> String {
        String::from(DEFAULT_NAME)
    }
    fn uid(&self) -> Uid {
        DEFAULT_ID
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn dependents(&self) -> Vec<Uid> {
        return vec![];
    }

    fn clone_box(&self) -> Box<dyn ComponentConfig> {
        Box::new(self.clone())
    }
}
