use std::collections::HashMap;
use std::iter::Map;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::thread;
use std::thread::JoinHandle;
use std::thread::current;

use crate::components::shares::StartableCompnent;
use crate::components::shares::{Component, ComponentLookupHelper};
use anyhow::Context;
use anyhow::Ok;
use anyhow::{Error, Result};
use gst::Element;
use gst::prelude::*;
use project_mapper_core::runtime_config::shared::Uid;
use project_mapper_core::runtime_config::{
    input::{InputComponentConfig, common::InputConfig},
    output::{OutputComponentConfig, common::OutputConfig, window::WindowConfig},
    shared::ComponentConfig,
};
use winit::event::Event;
use winit::event::WindowEvent;
use winit::event_loop::EventLoop;
use winit::event_loop::EventLoopBuilder;
use winit::event_loop::EventLoopProxy;
use winit::window::WindowBuilder;

// setup global state. While this could be done without the Option
// keep it to allow us to determine which component is the "main" one
// and will start threads/etc
#[derive(Clone)]
struct GlobalWindowState {
    pub event_loop_proxy: Option<EventLoopProxy<()>>,
    pub window_configs: HashMap<String, WindowConfig>,
}
static WINDOW_PROXY: LazyLock<Mutex<Option<GlobalWindowState>>> =
    LazyLock::new(|| Mutex::new(None));

struct WindowComponent {
    config: OutputComponentConfig,
    window_config: WindowConfig,

    // gst elements
    queue_element: Element,
    output_element: Element,

    // winit state
    is_main: bool,
    event_thread: Option<JoinHandle<Result<()>>>,

    // helpers
    has_setup: bool,
}

impl WindowComponent {
    // helper to initialize the global state and detect if this component
    // should be the main one
    fn initialize_global_state(&mut self) -> Result<()> {
        // if the current proxy is None we can assume ownership of the winit state. This component
        // will now become the primary "runable" window
        let mut global_state = WINDOW_PROXY.lock().or(Err(Error::msg(
            "Unable to aquire window proxy lock. Should not happen in normal operation",
        )))?;

        if let None = *global_state {
            let new_global_state = GlobalWindowState {
                event_loop_proxy: None,
                window_configs: HashMap::new(),
            };

            // update the global state with a new value
            *global_state = Some(new_global_state.clone());
            self.is_main = true;
        }

        // make sure the global state has our window config
        if let Some(state) = global_state.as_mut() {
            state
                .window_configs
                .insert(self.config.name(), self.window_config.clone());
        }

        Ok(())
    }

    fn run_event_loop() -> Result<()> {
        // construct the event loop and update the global proxy state
        let event_loop = EventLoopBuilder::new().build()?;

        // ControlFlow::Wait pauses the event loop if no events are available to process.
        // This is ideal for non-game applications that only update in response to user
        // input, and uses significantly less power/CPU time than ControlFlow::Poll.
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

        // Update the global state with a proxy reference. This allows other threads/components to send
        // us messages
        {
            let mut global_state = WINDOW_PROXY.lock().or(Err(Error::msg(
                "Unable to aquire window proxy lock. Should not happen in normal operation",
            )))?;
            if let Some(state) = global_state.as_mut() {
                state.event_loop_proxy = Some(event_loop.create_proxy());
            }
        }

        // next handle windows.... todo

        event_loop.run(move |event, event_loop_target| {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        // Stop the event loop
                        // ! Todo send event to parent
                        event_loop_target.exit();
                    }
                    WindowEvent::Resized(_new_size) => {
                        // Optionally handle resize
                    }
                    _ => (),
                },
                _ => (),
            }
        })?;

        Ok(())
    }
}

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

        let local_config = config.clone();
        if let OutputConfig::Window(window_config) = config.config {
            // get the current window config
            let window_config = window_config.clone();

            // construct the queue element which stops us from blocking this output
            let queue_name = format!("queue-{}", local_config.name());
            let queue_element = gst::ElementFactory::make("queue")
                .name(queue_name)
                .build()?;

            // construct the opengl image sink and ensure it's configured properly
            let output_element = gst::ElementFactory::make("glimagesink")
                .name(local_config.name())
                .build()?;
            // Disable sync to avoid blocking on display
            output_element.set_property("sync", &false);

            Ok(Self {
                config: local_config,
                window_config: window_config,

                output_element: output_element,
                queue_element: queue_element,
                has_setup: false,

                // create empty state later setup during setup
                event_thread: None,
                is_main: false,
            })
        } else {
            Err(Error::msg(
                "OutputConfig is not correct type for this component",
            ))
        }
    }

    // Run any post init setup functions
    fn setup(
        &mut self,
        pipeline: &gst::Pipeline,
        lookup_func: &dyn ComponentLookupHelper,
    ) -> Result<()> {
        if self.has_setup {
            return Ok(());
        }
        // start by setting up the correct proxy values and ensuring the global state is correct
        self.initialize_global_state()?;

        // Add both elements to the pipelines and sync status
        pipeline.add(&self.queue_element)?;
        pipeline.add(&self.output_element)?;
        self.queue_element.sync_state_with_parent()?;
        self.output_element.sync_state_with_parent()?;

        // Link elements
        self.queue_element.link(&self.output_element)?;

        // Fetch the compoennt that should be pointing to us
        if !lookup_func.has_uid(self.config.src_uid) {
            return Err(Error::msg(format!("Unknown Uid: {}", self.config.src_uid)));
        }
        let src_comp = lookup_func.lookup_and_setup(self.config.src_uid, pipeline, lookup_func);

        // link the desired source with the queue
        src_comp.element().link(&self.queue_element)?;

        // mark setup as complete so as to not rerun
        self.has_setup = true;
        Ok(())
    }

    // accessor functions
    fn element(&self) -> &Element {
        &self.queue_element
    }
    fn uid(&self) -> Uid {
        self.config.uid()
    }
}

impl StartableCompnent for WindowComponent {
    // Start this component
    fn start(&mut self) -> Result<()> {
        // if we're not main then do nothing
        if !self.is_main {
            return Ok(());
        }

        let window_thread = thread::spawn(WindowComponent::run_event_loop);
        self.event_thread = Some(window_thread);

        Ok(())
    }

    // Stop this component
    fn destroy(&mut self) -> Result<()> {
        // if we're not main then do nothing
        if !self.is_main {
            return Ok(());
        }
        Ok(())
    }
}
