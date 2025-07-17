use std::sync::mpsc;

use anyhow::Result;
use gst::prelude::*;
use project_mapper_core::runtime_config::RuntimeConfig;
use project_mapper_core::runtime_config::shared::{ComponentConfig, Uid};

use crate::components::comp_helper::ComponentHelper;
use crate::components::shared::ComponentLookupHelper;
use crate::types::message::RuntimeMessage;

pub struct Runtime {
    pub config: RuntimeConfig,
    pub component_helper: Box<dyn ComponentLookupHelper>,

    // message sender/reciever for runtime events
    pub message_sender: mpsc::Sender<RuntimeMessage>,
    message_reciever: mpsc::Receiver<RuntimeMessage>,
}

impl Runtime {
    pub fn new(
        config: RuntimeConfig,
        component_helper: Box<dyn ComponentLookupHelper>,
    ) -> Result<Self> {
        let (send, recv) = mpsc::channel();

        Ok(Self {
            config: config,
            component_helper: component_helper,
            // handle the message
            message_sender: send,
            message_reciever: recv,
        })
    }

    pub fn watch_for_events(self) -> Result<()> {
        for event in self.message_reciever.iter() {
            match event {
                RuntimeMessage::StopRuntime() => {}
            }
        }
        Ok(())
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
            self.component_helper
                .lookup_and_setup(output_uid, &pipeline)?;
        }

        // Start the pipeline
        pipeline.set_state(gst::State::Playing)?;

        // Next tell the component helper to start all components
        self.component_helper.start_and_run(&pipeline)?;

        // wait for events to exit I guess?
        // let (send, recv): (mpsc::Sender<RuntimeMessage>, Receiver<RuntimeMessage>) =
        //     mpsc::channel();
        // for event in recv.iter() {
        //     // do nothing for ever
        // }

        Ok(())
    }
}
