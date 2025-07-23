use core::time;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

use anyhow::{Context, Result};
use gst::prelude::*;
use project_mapper_core::runtime_config::RuntimeConfig;
use project_mapper_core::runtime_config::shared::{ComponentConfig, Uid};

use crate::components::runtime::DefaultRuntimeComponent;
use crate::components::shared::{ComponentFactory, ComponentLookupHelper};
use crate::types::message::RuntimeMessage;

pub struct Runtime {
    pub config: RuntimeConfig,
    pub component_factory: Box<dyn ComponentFactory>,
    pub component_helper: Box<dyn ComponentLookupHelper>,

    // message sender/reciever for runtime events
    pub message_sender: mpsc::Sender<RuntimeMessage>,
    message_reciever: Arc<Mutex<mpsc::Receiver<RuntimeMessage>>>,
}

impl Runtime {
    pub fn new(
        config: RuntimeConfig,
        component_factory: Box<dyn ComponentFactory>,
        component_helper: Box<dyn ComponentLookupHelper>,
    ) -> Result<Self> {
        let (send, recv) = mpsc::channel();

        // ensure the config is generally valid
        config
            .validate()
            .context("Failed to validate runtime config")?;

        Ok(Self {
            config: config,
            component_helper: component_helper,
            component_factory: component_factory,
            // handle the message
            message_sender: send,
            message_reciever: Arc::new(Mutex::new(recv)),
        })
    }

    // run the given runtime
    pub fn run(mut self) -> Result<()> {
        // start by initializing gst and the pipeline
        gst::init().context("Failed to initialize gst")?;
        let pipeline = gst::Pipeline::new();

        // next add all components to the comp helper. Make sure to track the output
        // components since those will be the root of our lookup_and_setup function
        let mut output_uids: Vec<Uid> = vec![];
        for input_config in &self.config.inputs {
            self.component_helper
                .create_and_insert_comp(input_config, self.component_factory.as_ref())
                .context(format!(
                    "failed to create input component: {}",
                    input_config.uid()
                ))?;
        }
        for effect_config in &self.config.effects {
            self.component_helper
                .create_and_insert_comp(effect_config, self.component_factory.as_ref())
                .context(format!(
                    "failed to create effect component: {}",
                    effect_config.uid()
                ))?;
        }
        for output_config in &self.config.outputs {
            self.component_helper
                .create_and_insert_comp(output_config, self.component_factory.as_ref())
                .context(format!(
                    "failed to create output component: {}",
                    output_config.uid()
                ))?;
            output_uids.push(output_config.uid());
        }

        // for each output uid call setup. No need to do this on other components since they
        // will work recursively
        for output_uid in output_uids {
            self.component_helper
                .lookup_and_setup(output_uid, &pipeline, self.message_sender.clone())
                .context(format!("failed to setup component: {}", output_uid))?;
        }

        // if there is no component that requires main then add the default runtime component.
        // this keeps the logic the same
        if !self.component_helper.has_main_requirement() {
            let default_config = DefaultRuntimeComponent::new_config()
                .context("Failed to contstruct default runtime component config")?;
            self.component_helper
                .create_and_insert_comp(&default_config, self.component_factory.as_ref())
                .context(format!("failed to create default runtime component"))?;
            self.component_helper
                .lookup_and_setup(
                    ComponentConfig::uid(&default_config),
                    &pipeline,
                    self.message_sender.clone(),
                )
                .context("Failed to setup default runtime component")?;
        }

        // Start the pipeline
        pipeline
            .set_state(gst::State::Playing)
            .context("Failed to start pipeline")?;

        // Next tell the component helper to start all components
        self.component_helper
            .start_or_resume(&pipeline)
            .context("Failed to start or resume components")?;

        // Then run the components
        loop {
            let message = self
                .component_helper
                .run(&pipeline, self.message_reciever.clone())
                .context("failed to run main component")?;

            match message {
                RuntimeMessage::ExitRuntime() => {
                    self.component_helper
                        .stop()
                        .context("Failed to stop components")?;

                    println!("Exiting runtime due to exit event: {:?}", message);
                    return Ok(());
                }
                RuntimeMessage::UpdateRuntime(_) => {}
            }
        }
    }
}
