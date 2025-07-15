use std::collections::HashMap;
use std::iter::Map;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::thread::current;

use crate::components::shares::{Component, ComponentLookupHelper};
use anyhow::Context;
use anyhow::{Error, Result};
use gst::Element;
use gst::prelude::*;
use project_mapper_core::runtime_config::{
    input::{InputComponentConfig, common::InputConfig},
    output::{OutputComponentConfig, common::OutputConfig, window::WindowConfig},
    shared::ComponentConfig,
};
use winit::event_loop::EventLoop;
use winit::event_loop::EventLoopBuilder;
use winit::event_loop::EventLoopProxy;
use winit::window::WindowBuilder;

struct WindowState {
    event_loop: EventLoop<()>,
}

#[derive(Clone)]
struct GlobalWindowState {
    pub event_loop_proxy: EventLoopProxy<()>,
    pub window_configs: HashMap<String, WindowConfig>,
}
static WINDOW_PROXY: LazyLock<Mutex<Option<GlobalWindowState>>> =
    LazyLock::new(|| Mutex::new(None));

struct WindowComponent {
    config: OutputComponentConfig,
    window_config: WindowConfig,
    element: Element,

    // winit state
    window_state: Option<WindowState>,
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

            // construct the opengl image sink and ensure it's configured properly
            let element = gst::ElementFactory::make("glimagesink")
                .name(local_config.name())
                .build()?;
            // Disable sync to avoid blocking on display
            element.set_property("sync", &false);

            Ok(Self {
                config: local_config,
                window_config: window_config,
                element: element,

                // create empty state later setup during setup
                window_state: None,
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
        // start by setting up the correct proxy values
        {
            // if the current proxy is None we can assume ownership of the winit state. This component
            // will now become the primary "runable" window
            let mut global_state = WINDOW_PROXY.lock().or(Err(Error::msg(
                "Unable to aquire window proxy lock. Should not happen in normal operation",
            )))?;

            if let None = *global_state {
                let event_loop = EventLoopBuilder::new().build()?;
                let window_state = WindowState {
                    event_loop: event_loop,
                };
                let new_global_state = GlobalWindowState {
                    event_loop_proxy: window_state.event_loop.create_proxy(),
                    window_configs: HashMap::new(),
                };

                // update the global state with a new value
                *global_state = Some(new_global_state.clone());
                self.window_state = Some(window_state);
            }

            // make sure the globa
            if let Some(state) = global_state.as_mut() {
                state
                    .window_configs
                    .insert(self.config.name(), self.window_config.clone());
            }
        }
        Ok(())
    }

    // accessor functions
    fn element(&self) -> &Element {
        &self.element
    }
}
