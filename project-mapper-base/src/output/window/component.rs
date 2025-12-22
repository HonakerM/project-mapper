use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;
use std::thread;

use crate::output::WindowConfig;
use crate::output::window::app::WindowAppHandler;
use crate::output::window::config::AvailableWindowConfig;
use crate::output::window::config::MonitorConfig;
use crate::output::window::state::GLOBAL_WINDOW_STATE;
use crate::output::window::state::PROXY_WINDOW_STATE;
use crate::output::window::state::ProxyWindowState;
use crate::output::window::state::WindowRequest;
use crate::output::window::state::WinitMessage;
use crate::output::window::state::WinitState;
use anyhow::Ok;
use anyhow::anyhow;
use anyhow::{Error, Result};
use log::info;
use project_mapper_core::runtime_config::output::OutputComponentConfig;
use project_mapper_core::runtime_config::shared::ComponentConfig;
use project_mapper_core::runtime_config::shared::Uid;
use project_mapper_core::types::openapi::OpenAPISchema;
use project_mapper_runtime::components::branch::BranchControl;
use project_mapper_runtime::components::runtime::DefaultRuntimeComponent;
use project_mapper_runtime::components::shared::{Component, ComponentLookupHelper};
use project_mapper_runtime::gst;
use project_mapper_runtime::gst::Element;
use project_mapper_runtime::gst::prelude::*;
use project_mapper_runtime::gst_video::prelude::*;
use project_mapper_runtime::types::message::RuntimeMessage;
use schemars::json_schema;
use winit::event_loop::EventLoop;
use winit::platform::pump_events::EventLoopExtPumpEvents;

pub struct WindowComponent {
    config: OutputComponentConfig,
    window_config: WindowConfig,

    // message sender for runtime events
    message_sender: Option<mpsc::Sender<RuntimeMessage>>,
    pipeline: Option<gst::Pipeline>,

    // gst elements
    branch: BranchControl,
    output_element: Element,

    // winit state
    is_main: bool,
}

impl WindowComponent {
    // helper to initialize the global state and detect if this component
    // should be the main one
    fn initialize_global_state(&mut self) -> Result<()> {
        // if the current proxy is None we can assume ownership of the winit state. This component
        // will now become the primary "runable" window
        let mut global_state = PROXY_WINDOW_STATE.lock().or(Err(Error::msg(
            "Unable to aquire window proxy lock. Should not happen in normal operation",
        )))?;

        if let None = *global_state {
            let mut new_global_state: ProxyWindowState = ProxyWindowState {
                event_loop_proxy: None,
                window_comps: HashSet::new(),
                has_main: true,
                available_config: AvailableWindowConfig::default(),
            };
            new_global_state.window_comps.insert(self.config.uid());

            // update the global state with a new value
            *global_state = Some(new_global_state.clone());
            self.is_main = true;
        } else if let Some(proxy_state) = global_state.as_mut() {
            proxy_state.window_comps.insert(self.config.uid());
        }

        Ok(())
    }

    fn initialize_event_loop(
        &mut self,
        pipeline: &gst::Pipeline,
        message_sender: mpsc::Sender<RuntimeMessage>,
    ) -> Result<()> {
        // construct the event loop and window mapping
        let event_loop: EventLoop<WinitMessage> = EventLoop::with_user_event().build()?;

        // ControlFlow::Wait pauses the event loop if no events are available to process.
        // This is ideal for non-game applications that only update in response to user
        // input, and uses significantly less power/CPU time than ControlFlow::Poll.
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

        // create all the required windows and update the global state
        {
            let mut global_state = PROXY_WINDOW_STATE.lock().or(Err(Error::msg(
                "Unable to aquire window proxy lock. Should not happen in normal operation",
            )))?;
            if let Some(state) = global_state.as_mut() {
                // update state with proxy
                state.event_loop_proxy = Some(event_loop.create_proxy());
            }
        }

        GLOBAL_WINDOW_STATE.replace(WinitState {
            handler: Some(WindowAppHandler::new(
                pipeline.clone(),
                message_sender.clone(),
                event_loop.create_proxy(),
            )),
            // don't create the message sender until we have it in run
            message_sender_thread: None,
            event_loop: Some(event_loop),
        });

        Ok(())
    }

    fn run_event_monitor(
        message_receiver: Arc<Mutex<mpsc::Receiver<RuntimeMessage>>>,
    ) -> Result<()> {
        loop {
            // use the default runtime's component manage events function to get the next event in a
            // controlled manor
            let event = DefaultRuntimeComponent::manage_events(message_receiver.clone())?;
            match event {
                event => {
                    // Fetch the window proxy so we can do something with it
                    let proxy_state = PROXY_WINDOW_STATE.lock().or(Err(Error::msg(
                        "Unable to aquire window proxy lock. Should not happen in normal operation",
                    )))?;

                    if let Some(state) = proxy_state.as_ref()
                        && let Some(proxy) = &state.event_loop_proxy
                    {
                        let is_exit = matches!(event, RuntimeMessage::ExitRuntime());
                        proxy
                            .send_event(WinitMessage::Runtime(event))
                            .map_err(|_| {
                                anyhow!("Unable to send message to event loop. It must be closed")
                            })?;
                        if is_exit {
                            return Ok(());
                        }
                    } else {
                        return Err(anyhow!("Unable to get proxt to get event"));
                    }
                }
            }
        }
    }

    fn run_event_loop(
        &self,
        message_sender: mpsc::Sender<RuntimeMessage>,
        message_receiver: std::sync::Arc<
            std::sync::Mutex<std::sync::mpsc::Receiver<RuntimeMessage>>,
        >,
    ) -> Result<WinitMessage> {
        if !self.is_main {
            return Err(Error::msg(
                "Unable to run event loop since this is not main",
            ));
        }
        let mut winit_state = GLOBAL_WINDOW_STATE.take();

        // if we haven't already started the event thread do so
        if winit_state.message_sender_thread.is_none() {
            let event_handle =
                thread::spawn(|| WindowComponent::run_event_monitor(message_receiver));
            winit_state.message_sender_thread = Some(event_handle);
        }

        let mut event_loop = winit_state.event_loop.take().ok_or(anyhow!(
            "Event loop does not exist which should never happen"
        ))?;
        let mut app_handler = winit_state.handler.take().ok_or(anyhow!(
            "App Handler does not exist which should never happen"
        ))?;

        while !app_handler.has_event() {
            event_loop.pump_app_events(None, &mut app_handler);
        }
        info!("Exiting event loop");

        let last_event = app_handler.get_next_event().unwrap();

        // if there was an exit error return it first
        if let Some(err) = app_handler.exit_err {
            app_handler.exit_err = None;
            return Err(err);
        }

        // after running replace the global state and event loop to retain references to the windows
        winit_state.event_loop = Some(event_loop);
        winit_state.handler = Some(app_handler);
        GLOBAL_WINDOW_STATE.set(winit_state);

        // get the exit event if it was created
        Ok(last_event)
    }

    fn update_window(&self, config: &WindowConfig, force_creation: bool) -> Result<()> {
        let winit_state_option = PROXY_WINDOW_STATE.lock().unwrap();
        if let Some(winit_state) = winit_state_option.as_ref() {
            if let Some(proxy) = &winit_state.event_loop_proxy {
                proxy.send_event(WinitMessage::UpdateWindow(WindowRequest {
                    element_name: self.config.name(),
                    element_uid: self.config.uid(),
                    config: config.clone(),
                }));
            }
        }

        if force_creation {
            println!("Updating window in {}", self.config.name());
            let mut global_state = GLOBAL_WINDOW_STATE.take();
            if let Some(event_loop) = &mut global_state.event_loop {
                if let Some(handler) = &mut global_state.handler {
                    event_loop.pump_app_events(None, handler);
                }
            }
            GLOBAL_WINDOW_STATE.set(global_state);
        }

        Ok(())
    }

    fn update_global_cfg(cfg: AvailableWindowConfig) {
        let mut winit_state_option = PROXY_WINDOW_STATE.lock().unwrap();
        if let Some(winit_state) = winit_state_option.as_mut() {
            winit_state.available_config = cfg;
        }
    }
    fn openapi_schema() -> OpenAPISchema {
        let winit_state_option = PROXY_WINDOW_STATE.lock().unwrap();
        if let Some(winit_state) = winit_state_option.as_ref() {
            winit_state.available_config.openapi_schema()
        } else {
            AvailableWindowConfig::default().openapi_schema()
        }
    }
}

#[project_mapper_macro::output_component(config={WindowConfig::default()}, schema={WindowComponent::openapi_schema().to_json_value()}, requires_refresh={true})]
impl Component for WindowComponent {
    // runtime lifecycle functions
    // Construct object
    fn new(unknown_config: &dyn ComponentConfig) -> Result<WindowComponent> {
        // parse config and ensure it's correct types
        let config: OutputComponentConfig = match unknown_config
            .as_any()
            .downcast_ref::<OutputComponentConfig>()
        {
            Some(b) => Ok(b.clone()),
            None => Err(Error::msg(
                "ComponentConfig can not be typed to OutputComponentConfig",
            )),
        }?;

        // load the config
        let window_config = match config.config.as_any().downcast_ref::<WindowConfig>() {
            Some(b) => Ok(b.clone()),
            None => Err(anyhow!("WindowComponentConfig is not WindowConfig")),
        }?;

        let branch = BranchControl::new(config.name(), true, false)?;
        let output_element = gst::ElementFactory::make("glimagesink")
            .name(config.name())
            .build()?;

        output_element.set_property("sync", &true);
        let mut window_component = Self {
            config: config,
            window_config: window_config,

            branch: branch,
            output_element: output_element,

            message_sender: None,
            pipeline: None,

            // default to not main. This will be configured during initialize_global_state
            is_main: false,
        };
        window_component.initialize_global_state()?;

        Ok(window_component)
    }

    // Run any post init setup functions
    fn setup(
        &mut self,
        pipeline: &gst::Pipeline,
        message_sender: mpsc::Sender<RuntimeMessage>,
    ) -> Result<()> {
        // copy the message sender into the object
        self.message_sender = Some(message_sender.clone());
        self.pipeline = Some(pipeline.clone());

        // Add both elements to the pipelines and sync status
        pipeline.add(&self.output_element)?;
        self.output_element.sync_state_with_parent()?;
        // construct branch and wrap
        self.branch.add_to_pipeline(pipeline)?;
        self.branch.link_wrapped(&self.output_element)?;

        {
            let winit_state = GLOBAL_WINDOW_STATE.take();
            if let None = winit_state.event_loop {
                // initialize all the window elements
                println!("Initializing event loop in {}", self.config.name());
                self.initialize_event_loop(&pipeline, message_sender.clone())?;
            } else {
                GLOBAL_WINDOW_STATE.set(winit_state);
            }
        }
        self.update_window(&self.window_config, true)?;

        // mark setup as complete so as to not rerun
        Ok(())
    }

    fn update(&mut self, config: &dyn ComponentConfig) -> Result<()> {
        // parse config and ensure it's correct types
        let config: OutputComponentConfig =
            match config.as_any().downcast_ref::<OutputComponentConfig>() {
                Some(b) => Ok(b.clone()),
                None => Err(Error::msg(
                    "ComponentConfig can not be typed to OutputComponentConfig",
                )),
            }?;

        // load the config
        let window_config = match config.config.as_any().downcast_ref::<WindowConfig>() {
            Some(b) => Ok(b.clone()),
            None => Err(anyhow!("WindowComponentConfig is not WindowConfig")),
        }?;
        self.config = config;
        self.window_config = window_config;
        self.update_window(&self.window_config, false)?;

        Ok(())
    }
    // accessor functions
    fn input_element(&self) -> Option<&Element> {
        self.branch.get_input()
    }
    fn output_element(&self) -> Option<&Element> {
        None
    }

    fn uid(&self) -> Uid {
        self.config.uid()
    }

    // Run this component
    fn run(
        &self,
        message_broker: std::sync::Arc<std::sync::Mutex<std::sync::mpsc::Receiver<RuntimeMessage>>>,
    ) -> Result<RuntimeMessage> {
        if !self.is_main {
            return Err(anyhow!(
                "Component is not main window. run should not have been called"
            ));
        }

        let message_sender = if let Some(sender) = &self.message_sender {
            Ok(sender)
        } else {
            Err(anyhow!(
                "Somehow window component did not get message sender"
            ))
        }?;

        // if we're main then run the real event loop!
        loop {
            let msg = self.run_event_loop(message_sender.clone(), message_broker.clone())?;
            match msg {
                WinitMessage::Runtime(msg) => {
                    return Ok(msg);
                }
                WinitMessage::AvailableConfig(cfg) => {
                    WindowComponent::update_global_cfg(cfg);
                }
                _ => {}
            }
        }

        /*
        // if we exited the event loop we also must have exited the event watcher...
         */
    }

    // Stop this component
    fn destroy(&mut self) -> Result<()> {
        // Unlink our components and destory them
        BranchControl::unlink_and_destory(&self.output_element)?;
        self.branch.destory()?;

        // Start by destorying our window
        GLOBAL_WINDOW_STATE.with_borrow_mut(|state| {
            if let Some(handler) = &mut state.handler {
                handler.destory_window(&self.config.uid());
            }
        });

        // destory our proxy reference
        let mut proxy_state = PROXY_WINDOW_STATE
            .lock()
            .or(Err(Error::msg(
                "Unable to aquire window proxy lock. Should not happen in normal operation",
            )))
            .unwrap();
        if let Some(proxy_state) = proxy_state.as_mut() {
            proxy_state.window_comps.remove(&self.config.uid);
            if self.is_main {
                proxy_state.has_main = false;
            }
        }
        Ok(())
    }

    // only the main window requires the main thread
    fn requires_main(&mut self) -> bool {
        if self.is_main {
            return true;
        } else {
            let mut global_state = PROXY_WINDOW_STATE
                .lock()
                .or(Err(Error::msg(
                    "Unable to aquire window proxy lock. Should not happen in normal operation",
                )))
                .unwrap();
            if let Some(proxy_state) = global_state.as_mut() {
                if !proxy_state.has_main {
                    self.is_main = true;
                    return true;
                }
            }
        }
        return self.is_main;
    }
}
