use core::time;
use std::sync::{Arc, LazyLock, Mutex, mpsc};
use std::thread;

use anyhow::{Context, Result, anyhow};
use gst::prelude::*;
use project_mapper_core::runtime_config::RuntimeConfig;
use project_mapper_core::runtime_config::shared::{ComponentConfig, Uid};

use crate::components::runtime::DefaultRuntimeComponent;
use crate::components::shared::{ComponentFactory, ComponentLookupHelper};
use crate::receivers::receiver::start_receiver;
use crate::types::message::RuntimeMessage;

static GLOBAL_RUNTIME_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

pub struct Runtime {
    pub component_factory: Box<dyn ComponentFactory>,
    pub component_helper: Box<dyn ComponentLookupHelper>,

    // message sender/reciever for runtime events
    pub message_sender: mpsc::Sender<RuntimeMessage>,
    message_reciever: Arc<Mutex<mpsc::Receiver<RuntimeMessage>>>,
}

impl Runtime {
    pub fn new(
        component_factory: Box<dyn ComponentFactory>,
        component_helper: Box<dyn ComponentLookupHelper>,
    ) -> Result<Self> {
        let (send, recv) = mpsc::channel();

        Ok(Self {
            component_helper: component_helper,
            component_factory: component_factory,
            // handle the message
            message_sender: send,
            message_reciever: Arc::new(Mutex::new(recv)),
        })
    }

    // run the given runtime
    pub fn run(mut self, config: Arc<Mutex<RuntimeConfig>>) -> Result<()> {
        // Before doing literally anything. Initialize GST
        gst::init().context("Failed to initialize gst")?;

        // aquire the global lock to ensure only one runtime can run at a time
        let _unused_lock = GLOBAL_RUNTIME_LOCK
            .lock()
            .map_err(|e| {
                anyhow!(
                    "Recieved poision error while aquireing global runtime lock: {}",
                    e.to_string()
                )
            })
            .context("Failed to aquire global runtime lock")?;

        // track the output uids so we know what to setup
        let mut output_uids: Vec<Uid> = vec![];
        // setup all the components based on the config
        {
            let local_config = config
                .lock()
                .map_err(|e| {
                    anyhow!(
                        "Recieved poision error while aquireing config lock: {}",
                        e.to_string()
                    )
                })
                .context("Failed to aquire config lock")?;

            // ensure the config is generally valid
            local_config
                .validate()
                .context("Failed to validate runtime config")?;

            // next add all components to the comp helper. Make sure to track the output
            // components since those will be the root of our lookup_and_setup function
            for input_config in &local_config.inputs {
                self.component_helper
                    .create_and_insert_comp(input_config, self.component_factory.as_ref())
                    .context(format!(
                        "failed to create input component: {}",
                        input_config.uid()
                    ))?;
            }
            for effect_config in &local_config.effects {
                self.component_helper
                    .create_and_insert_comp(effect_config, self.component_factory.as_ref())
                    .context(format!(
                        "failed to create effect component: {}",
                        effect_config.uid()
                    ))?;
            }
            for output_config in &local_config.outputs {
                self.component_helper
                    .create_and_insert_comp(output_config, self.component_factory.as_ref())
                    .context(format!(
                        "failed to create output component: {}",
                        output_config.uid()
                    ))?;
                output_uids.push(output_config.uid());
            }
        }

        // create the pipeline
        let pipeline = gst::Pipeline::new();

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

        // start the receiver threads
        let mut receiver_handle = start_receiver(self.message_sender.clone(), config.clone())?;

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

                    receiver_handle.cancel()?;

                    println!("Exiting runtime due to exit event: {:?}", message);
                    return Ok(());
                }
                RuntimeMessage::UpdateRuntime(_) => {}
            }
        }
    }
}
