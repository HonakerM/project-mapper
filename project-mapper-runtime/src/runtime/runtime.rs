use std::sync::{Arc, Mutex, mpsc};

use anyhow::Result;
use gst::prelude::*;
use project_mapper_core::runtime_config::RuntimeConfig;
use project_mapper_core::runtime_config::shared::{ComponentConfig, Uid};

use crate::components::runtime::DefaultRuntimeComponent;
use crate::components::shared::ComponentLookupHelper;
use crate::types::message::RuntimeMessage;

pub struct Runtime {
    pub config: RuntimeConfig,
    pub component_helper: Box<dyn ComponentLookupHelper>,

    // message sender/reciever for runtime events
    pub message_sender: mpsc::Sender<RuntimeMessage>,
    message_reciever: Arc<Mutex<mpsc::Receiver<RuntimeMessage>>>,
}

impl Runtime {
    pub fn new(
        config: RuntimeConfig,
        component_helper: Box<dyn ComponentLookupHelper>,
    ) -> Result<Self> {
        let (send, recv) = mpsc::channel();

        // ensure the config is generally valid
        config.validate()?;

        Ok(Self {
            config: config,
            component_helper: component_helper,
            // handle the message
            message_sender: send,
            message_reciever: Arc::new(Mutex::new(recv)),
        })
    }

    // run the given runtime
    pub fn run(mut self) -> Result<()> {
        // start by initializing gst and the pipeline
        gst::init()?;
        let pipeline = gst::Pipeline::new();

        // next add all components to the comp helper. Make sure to track the output
        // components since those will be the root of our lookup_and_setup function
        let mut output_uids: Vec<Uid> = vec![];
        for input_config in &self.config.inputs {
            self.component_helper.create_and_insert_comp(input_config)?;
        }
        for effect_config in &self.config.effects {
            self.component_helper
                .create_and_insert_comp(effect_config)?;
        }
        for output_config in &self.config.outputs {
            self.component_helper
                .create_and_insert_comp(output_config)?;
            output_uids.push(output_config.uid());
        }

        // for each output uid call setup. No need to do this on other components since they
        // will work recursively
        for output_uid in output_uids {
            self.component_helper.lookup_and_setup(
                output_uid,
                &pipeline,
                self.message_sender.clone(),
            )?;
        }

        // if there is no component that requires main then add the default runtime component.
        // this keeps the logic the same
        if !self.component_helper.has_main_requirement() {
            let default_config = DefaultRuntimeComponent::new_config()?;
            self.component_helper
                .create_and_insert_comp(&default_config)?;
            self.component_helper.lookup_and_setup(
                ComponentConfig::uid(&default_config),
                &pipeline,
                self.message_sender.clone(),
            )?;
        }

        // Start the pipeline
        pipeline.set_state(gst::State::Playing)?;

        // Next tell the component helper to start all components
        self.component_helper.start_or_resume(&pipeline)?;

        // Then run the components
        let message = self
            .component_helper
            .run(&pipeline, self.message_reciever)?;

        match message {
            RuntimeMessage::ExitRuntime() => {
                self.component_helper.stop()?;

                println!("Exiting runtime due to exit event: {:?}", message);
            }
        }

        // wait for events to exit I guess?
        // let (send, recv): (mpsc::Sender<RuntimeMessage>, Receiver<RuntimeMessage>) =
        //     mpsc::channel();
        // for event in recv.iter() {
        //     // do nothing for ever
        // }

        Ok(())
    }
}
