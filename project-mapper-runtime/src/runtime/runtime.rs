use core::time;
use std::any::type_name_of_val;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, LazyLock, Mutex, mpsc};
use std::thread;

use anyhow::{Context, Result, anyhow};
use gst::{DebugGraphDetails, StateChangeSuccess, prelude::*};
use project_mapper_core::runtime_config::output::OutputComponentConfig;
use project_mapper_core::runtime_config::shared::{ComponentConfig, Uid};
use project_mapper_core::runtime_config::{RuntimeConfig, output};

use crate::components::runtime::DefaultRuntimeComponent;
use crate::components::shared::{ComponentFactory, ComponentLookupHelper};
use crate::receivers::receiver::start_receiver;
use crate::runtime::configure::{configure_components, update_components};
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
        // initialize gstreamer
        gst::init()?;

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

        // create the pipeline
        let pipeline = gst::Pipeline::new();

        // configure all components
        self.configure_components(&config, &pipeline)?;
        info!("Configured components");

        // start the receiver threads
        let mut receiver_handle = start_receiver(self.message_sender.clone(), config.clone())?;

        let cloned_pipeline = pipeline.clone();
        let mut pipeline_monitor_handle =
            thread::spawn(move || Runtime::monitor_pipeline_events(cloned_pipeline));

        //let local_pipeline = pipeline.clone();
        //thread::spawn(|| Runtime::monitor_pipeline_events(local_pipeline));

        // Start the pipeline
        pipeline
            .set_state(gst::State::Playing)
            .context("Failed to start pipeline")?;

        // Next tell the component helper to start all components
        self.component_helper
            .resume()
            .context("Failed to start or resume components")?;

        // right after starting the pipeline export it to a file
        pipeline.debug_to_dot_file(DebugGraphDetails::all(), Path::new("./pipeline.dot"));

        // Then run the components
        loop {
            let message = self
                .component_helper
                .run(self.message_reciever.clone())
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


    fn configure_components(&mut self, current_config: &Arc<Mutex<RuntimeConfig>>, pipeline: &gst::Pipeline) -> Result<()> {
        let local_config = current_config.lock().unwrap();
        configure_components(&local_config, pipeline, &mut self.component_helper, &self.component_factory, self.message_sender.clone())?;
        Ok(())
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
            for deleted_uid in &change_tracker.deletes {
                self.component_helper.destroy_comp(deleted_uid)?;
            }

            // update the stored config
            *locked_current_config = new_config;

            change_tracker
        };
        update_components(change_tracker, pipeline, &mut self.component_helper, &self.component_factory, self.message_sender.clone())?;

        Ok(())
    }

    fn monitor_pipeline_events(pipeline: gst::Pipeline) {
        let bus = pipeline.bus().unwrap();
        for msg in bus.iter_timed(gst::ClockTime::NONE) {
            match msg.view() {
                gst::MessageView::Error(err) => {
                    warn!(
                        "Error from {}: {} ({:?})",
                        err.src()
                            .map(|s| s.path_string())
                            .unwrap_or_else(|| "None".to_string().into()),
                        err.error(),
                        err.debug()
                    );
                }
                gst::MessageView::StateChanged(state) => {
                    // info!(
                    // "State changed: {} -> {:?}",
                    // state.src().map(|s| s.path_string()).unwrap(),
                    // state.current()
                    // );
                }
                gst::MessageView::NeedContext(need_context_msg) => {
                    println!("got need context message {:?}", need_context_msg);
                    // if need_context_msg.context_type() == "gst.gl.GLDisplay" {
                    //     let obj: &gst::Object = msg.src().unwrap(); // element requesting the GLDisplay
                    //     // Match to the right GLDisplay
                    //     if obj.name() == local_element_name {
                    //         println!("Has name match {:?}", local_element_name);
                    //         if let Ok(element) = obj.clone().dynamic_cast::<Element>() {
                    //             println!("Setting context in {:?}", local_element_name);
                    //             element.set_context(&context);
                    //         }
                    //     }
                    // }
                }
                _ => {}
                msg => {
                    //info!(
                    //    "Receieved unknown message (type: {}): {:?}",
                    //    type_name_of_val(&msg),
                    //    msg
                    //)
                }
                _ => {}
            }
        }
    }
}
