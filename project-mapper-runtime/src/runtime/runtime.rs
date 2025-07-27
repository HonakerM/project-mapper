use core::time;
use std::any::type_name_of_val;
use std::path::Path;
use std::sync::{Arc, LazyLock, Mutex, mpsc};
use std::thread;

use anyhow::{Context, Result, anyhow};
use gst::{DebugGraphDetails, StateChangeSuccess, prelude::*};
use project_mapper_core::runtime_config::RuntimeConfig;
use project_mapper_core::runtime_config::output::OutputComponentConfig;
use project_mapper_core::runtime_config::shared::{ComponentConfig, Uid};

use crate::components::runtime::DefaultRuntimeComponent;
use crate::components::shared::{ComponentFactory, ComponentLookupHelper};
use crate::receivers::receiver::start_receiver;
use crate::types::message::RuntimeMessage;
use log::{info, warn};

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

        // setup the ctrlc handler early on to ensure we always capture the signal
        let local_send = send.clone();
        ctrlc::set_handler(move || {
            local_send
                .send(RuntimeMessage::ExitRuntime())
                .expect("Unable to send exit event. Panicing");
        })
        .context("Error setting Ctrl-C handler")?;

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

        let component_configs = {
            config
                .lock()
                .map_err(|e| anyhow!("Unable to aquire config lock due to poison: {:#}", e))?
                .gather_configs()
        };

        // track the output uids so we know what to setup
        let output_uids: Vec<Uid> = self
            .create_or_update_components(&config, component_configs)
            .context("Unable to create components")?;

        // create the pipeline
        let pipeline = gst::Pipeline::new();

        // for each output uid call setup. No need to do this on other components since they
        // will work recursively
        for output_uid in output_uids {
            self.component_helper
                .lookup_and_setup(output_uid, &pipeline, self.message_sender.clone())
                .context(format!("failed to setup component: {}", output_uid))?;
        }

        // start the receiver threads
        let mut receiver_handle = start_receiver(self.message_sender.clone(), config.clone())?;
        //let local_pipeline = pipeline.clone();
        //thread::spawn(|| Runtime::monitor_pipeline_events(local_pipeline));

        // right before starting the pipeline export it to a file
        pipeline.debug_to_dot_file(DebugGraphDetails::all(), Path::new("./pipeline.dot"));

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
                    // First pause and then destroy all components
                    self.component_helper
                        .pause()
                        .context("Failed to stop components")?;
                    self.component_helper
                        .destory()
                        .context("Failed to destroy components")?;

                    // stop the receivers
                    receiver_handle
                        .cancel()
                        .context("Failed to stop receiver handle")?;

                    info!("Exiting runtime due to exit event: {:?}", message);
                    return Ok(());
                }
                RuntimeMessage::UpdateRuntime(new_config) => {
                    info!("Running update with new config: {:?}", config);
                    // stop the pipeline before updating
                    //pipeline.set_state(gst::State::Paused)?;

                    self.update_runtime(&pipeline, &config, new_config)?;

                    // restart the pipeline
                    //pipeline.set_state(gst::State::Playing)?;
                    info!("Completed update logic");
                }
            }
        }
    }

    fn update_runtime(
        &mut self,
        pipeline: &gst::Pipeline,
        current_config: &Arc<Mutex<RuntimeConfig>>,
        new_config: RuntimeConfig,
    ) -> Result<()> {
        let change_tracker = {
            // start by locking the stored runtime
            let mut locked_current_config = current_config
                .lock()
                .map_err(|e| anyhow!("Unable to lock config for update {:#}", e))?;

            // ensure we destory the components before running create/update
            let change_tracker = locked_current_config.gather_config_changes(&new_config)?;
            for deleted_configs in &change_tracker.deletes {
                self.component_helper.destroy_comp(&deleted_configs.uid())?;
            }

            // update the stored config
            *locked_current_config = new_config;

            change_tracker
        };

        info!("Updated configs {:?}", change_tracker.updates);
        info!("Deleted configs {:?}", change_tracker.deletes);

        // Create or update the remaining components
        let updated_output_uids =
            self.create_or_update_components(current_config, change_tracker.updates)?;

        // for each output uid call setup. No need to do this on other components since they
        // will work recursively
        for output_uid in updated_output_uids {
            self.component_helper
                .lookup_and_setup(output_uid, &pipeline, self.message_sender.clone())
                .context(format!("failed to setup component: {}", output_uid))?;
        }
        Ok(())
    }

    fn create_or_update_components(
        &mut self,
        config: &Arc<Mutex<RuntimeConfig>>,
        component_configs: Vec<Box<dyn ComponentConfig>>,
    ) -> Result<Vec<Uid>> {
        // if the component helper contains the default component then remove it before creating
        // or updating components. This ensures if we add a component that could be main the default
        // runtime doesn't affect it
        if self
            .component_helper
            .contains_comp(&DefaultRuntimeComponent::get_default_uid())
        {
            self.component_helper
                .destroy_comp(&DefaultRuntimeComponent::get_default_uid())?;
        }

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

            for config in component_configs {
                self.component_helper
                    .create_or_update(config.as_ref(), self.component_factory.as_ref())
                    .context(format!(
                        "failed to create or update component: {}",
                        config.uid()
                    ))?;

                // parse config to check if its an output type
                if let Some(__) = config.as_any().downcast_ref::<OutputComponentConfig>() {
                    output_uids.push(config.uid());
                }
            }
        }

        // if there is no component that requires main then add the default runtime component.
        // this keeps the logic the same
        if !self.component_helper.has_main_requirement() {
            let default_config = DefaultRuntimeComponent::new_config()
                .context("Failed to contstruct default runtime component config")?;
            self.component_helper
                .create_or_update(&default_config, self.component_factory.as_ref())
                .context(format!("failed to create default runtime component"))?;
            output_uids.push(default_config.uid());
        }

        Ok(output_uids)
    }

    fn monitor_pipeline_events(pipeline: gst::Pipeline) {
        let bus = pipeline.bus().unwrap();
        for msg in bus.iter_timed(gst::ClockTime::NONE) {
            match msg.view() {
                gst::MessageView::Error(err) => {
                    info!(
                        "Error from {}: {} ({:?})",
                        err.src()
                            .map(|s| s.path_string())
                            .unwrap_or_else(|| "None".to_string().into()),
                        err.error(),
                        err.debug()
                    );
                }
                gst::MessageView::StateChanged(state) => {
                    info!(
                        "State changed: {} -> {:?}",
                        state.src().map(|s| s.path_string()).unwrap(),
                        state.current()
                    );
                }
                msg => {
                    info!(
                        "Receieved unknown message (type: {}): {:?}",
                        type_name_of_val(&msg),
                        msg
                    )
                }
                _ => {}
            }
        }
    }
}
