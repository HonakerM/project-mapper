use core::time;
use std::sync::{Arc, LazyLock, Mutex, mpsc};
use std::thread;

use anyhow::{Context, Result, anyhow};
use gst::prelude::*;
use project_mapper_core::runtime_config::RuntimeConfig;
use project_mapper_core::runtime_config::shared::{ComponentConfig, Uid};

use crate::components::runtime::DefaultRuntimeComponent;
use crate::components::shared::{ComponentFactory, ComponentLookupHelper};
use crate::receivers::receiver::run_receiver;
use crate::types::message::RuntimeMessage;

static GLOBAL_RUNTIME_CONFIG: LazyLock<Arc<Mutex<Option<RuntimeConfig>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(None)));

pub struct Runtime {
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

        // update the global config with the one provided
        {
            if !GLOBAL_RUNTIME_CONFIG
                .lock()
                .map_err(|_| anyhow!("Unable to aquire global runtime lock"))?
                .is_none()
            {
                return Err(anyhow!(
                    "GLOBAL_RUNTIME_CONFIG has already been set. Can not have multiple runtimes running in the same process"
                ));
            }
        }
        Runtime::set_config(config)?;

        Ok(Self {
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
        let local_config = Runtime::get_config()?;
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
        thread::spawn(move || {
            run_receiver(self.message_sender.clone(), Arc::new(Runtime::get_config))
        });

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

    fn get_config() -> Result<RuntimeConfig> {
        let runtime_config_option = GLOBAL_RUNTIME_CONFIG
            .lock()
            .map_err(|_| anyhow!(("Unable to acquire global runtime lock")))?;

        match runtime_config_option.as_ref() {
            None => Err(anyhow!("Unable to locate global runtime config")),
            Some(config) => Ok(config.clone()),
        }
    }

    fn set_config(config: RuntimeConfig) -> Result<()> {
        let mut runtime_config_option = GLOBAL_RUNTIME_CONFIG
            .lock()
            .map_err(|_| anyhow!(("Unable to acquire global runtime lock")))?;
        runtime_config_option.replace(config);
        Ok(())
    }
}
