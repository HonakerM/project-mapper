use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::Mutex;

use crate::components::shared::{Component, ComponentLookupHelper};
use crate::utils::winit::get_monitor_by_name;
use crate::utils::winit::get_video_mode_for_config;
use anyhow::Context;
use anyhow::Ok;
use anyhow::{Error, Result};
use gst::Element;
use gst::prelude::*;
use gst_video::prelude::*;
use project_mapper_core::runtime_config::output::window::WindowMode;
use project_mapper_core::runtime_config::shared::Uid;
use project_mapper_core::runtime_config::{
    output::{OutputComponentConfig, common::OutputConfig, window::WindowConfig},
    shared::ComponentConfig,
};
use raw_window_handle::HasWindowHandle;
use raw_window_handle::RawWindowHandle;
use winit::event::Event;
use winit::event::WindowEvent;
use winit::event_loop::EventLoop;
use winit::event_loop::EventLoopBuilder;
use winit::event_loop::EventLoopProxy;
use winit::window::Window;
use winit::window::WindowBuilder;

// helper struct to store information about winit. This
// will only be held by the main component
struct WinitState {
    event_loop: EventLoop<()>,
    // needed to keep reference to a window
    _windows: HashMap<Uid, Window>,
}
// use thread local window state since we only ever need it on the main thread
thread_local! {
    static GLOBAL_WINDOW_STATE: RefCell<Option<WinitState>> = RefCell::new(None);
}

// Struct for components to request a window with
#[derive(Clone)]
struct WindowRequest {
    pub element_name: String,
    pub element_uid: Uid,
    pub config: WindowConfig,
}

// setup global state. While this could be done without the Option
// keep it to allow us to determine which component is the "main" one
// and will start threads/etc
#[derive(Clone)]
struct ProxyWindowState {
    pub event_loop_proxy: Option<EventLoopProxy<()>>,
    pub window_configs: HashMap<String, WindowRequest>,
}
static PROXY_WINDOW_STATE: LazyLock<Mutex<Option<ProxyWindowState>>> =
    LazyLock::new(|| Mutex::new(None));

pub struct WindowComponent {
    config: OutputComponentConfig,
    window_config: WindowConfig,

    // gst elements
    queue_element: Element,
    output_element: Element,

    // winit state
    is_main: bool,

    // helpers
    has_setup: bool,
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
            let new_global_state: ProxyWindowState = ProxyWindowState {
                event_loop_proxy: None,
                window_configs: HashMap::new(),
            };

            // update the global state with a new value
            *global_state = Some(new_global_state.clone());
            self.is_main = true;
        }

        // make sure the global state has our window config
        if let Some(state) = global_state.as_mut() {
            state.window_configs.insert(
                self.config.name(),
                WindowRequest {
                    element_uid: self.config.uid(),
                    element_name: self.config.name(),
                    config: self.window_config.clone(),
                },
            );
        }

        Ok(())
    }

    fn initialize_window_elements(&mut self, pipeline: &gst::Pipeline) -> Result<()> {
        // construct the event loop and window mapping
        let event_loop = EventLoopBuilder::new().build()?;
        let mut windows: HashMap<Uid, Window> = HashMap::new();

        // ControlFlow::Wait pauses the event loop if no events are available to process.
        // This is ideal for non-game applications that only update in response to user
        // input, and uses significantly less power/CPU time than ControlFlow::Poll.
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

        // create all the required windows and update the global state
        let mut global_state = PROXY_WINDOW_STATE.lock().or(Err(Error::msg(
            "Unable to aquire window proxy lock. Should not happen in normal operation",
        )))?;
        if let Some(state) = global_state.as_mut() {
            // update state with proxy
            state.event_loop_proxy = Some(event_loop.create_proxy());

            // while we have the lock setup all windows
            for window_request in state.window_configs.values() {
                // start by building the window
                // ! TODO actually use the window config
                let window = WindowBuilder::new()
                    .with_title(window_request.element_name.clone())
                    .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
                    .build(&event_loop)?;

                // get the gst element as a video overlay
                let element = pipeline
                    .by_name(&window_request.element_name)
                    .with_context(|| {
                        format!(
                            "Element with name '{}' not found in pipeline",
                            window_request.element_name
                        )
                    })?;
                let overlay = element
                    .dynamic_cast::<gst_video::VideoOverlay>()
                    .map_err(|_| anyhow::anyhow!("Failed to cast element to VideoOverlay"))?;

                // Obtain the raw window handle from winit
                let raw_handle = window.window_handle().unwrap().as_raw();

                // Extract platform-specific handle ID
                let handle_id = match raw_handle {
                    RawWindowHandle::Xlib(h) => h.window as usize,
                    RawWindowHandle::Wayland(h) => h.surface.as_ptr() as usize,
                    RawWindowHandle::Win32(h) => h.hwnd.get() as usize,
                    RawWindowHandle::AppKit(h) => h.ns_view.as_ptr() as usize,
                    _ => panic!("Unsupported platform: cannot get raw window handle"),
                };

                // set the gstreamer element to output to this handle!
                unsafe {
                    overlay.set_window_handle(handle_id);
                }

                // configure the window based on the config
                WindowComponent::configure_window(
                    &event_loop,
                    &window,
                    window_request.config.clone(),
                )?;

                // hold onto window ref
                windows.insert(window_request.element_uid, window);
            }
        }

        GLOBAL_WINDOW_STATE.replace(Some(WinitState {
            event_loop: event_loop,
            _windows: windows,
        }));

        Ok(())
    }

    fn configure_window(
        event_loop: &EventLoop<()>,
        window: &Window,
        config: WindowConfig,
    ) -> Result<()> {
        match &config.mode {
            WindowMode::Windowed {} => {}
            WindowMode::Borderless { name } => {
                let monitor_handle = get_monitor_by_name(event_loop, name.clone())?;
                window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(Some(
                    monitor_handle,
                ))));
            }
            WindowMode::Exclusive { config } => {
                let video_mode = get_video_mode_for_config(event_loop, config)?;
                window.set_fullscreen(Some(winit::window::Fullscreen::Exclusive(video_mode)));
            }
        }

        Ok(())
    }

    fn run_event_loop(&self) -> Result<()> {
        if !self.is_main {
            return Err(Error::msg(
                "Unable to run event loop since this is not main",
            ));
        }
        let winit_state = GLOBAL_WINDOW_STATE.take().ok_or(Error::msg(
            "Unable to run event loop since this is not main. We should have a window state",
        ))?;
        winit_state
            .event_loop
            .run(move |event, event_loop_target| {
                match event {
                    Event::WindowEvent { event, .. } => match event {
                        WindowEvent::CloseRequested => {
                            println!("close requestted");
                            // Stop the event loop
                            // ! Todo send event to parent
                            event_loop_target.exit();
                        }
                        _msg => {
                            //println!("other message {:?}", msg)
                        }
                    },
                    _ => (),
                }
            })?;
        return Ok(());
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

        // load the config
        let local_config = config.clone();
        if let OutputConfig::Window(window_config) = config.config {
            // get the current window config
            let window_config = window_config.clone();

            // construct the queue sink
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

            let mut window_component = Self {
                config: local_config,
                window_config: window_config,

                queue_element: queue_element,
                output_element: output_element,
                has_setup: false,

                // default to not main. This will be configured during initialize_global_state
                is_main: false,
            };
            window_component.initialize_global_state()?;
            Ok(window_component)
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

        // Add both elements to the pipelines and sync status
        pipeline.add(&self.queue_element)?;
        pipeline.add(&self.output_element)?;
        self.queue_element.sync_state_with_parent()?;
        self.output_element.sync_state_with_parent()?;

        // If we're the main function than recursively call setup on all available window references
        // this ensures all pipeline elements that require a window have been added to the pipeline
        // ! Note this must come after adding the components to the pipeline:

        if self.is_main {
            // only lock global state while gathering components to initialize
            let mut comp_ids_to_init: Vec<Uid> = vec![];
            {
                let global_state = PROXY_WINDOW_STATE.lock().or(Err(Error::msg(
                    "Unable to aquire window proxy lock. Should not happen in normal operation",
                )))?;
                if let Some(state) = global_state.as_ref() {
                    for request in state.window_configs.values() {
                        // we don't want to re-init this component as that can cause issues
                        if request.element_uid != self.uid() {
                            comp_ids_to_init.push(request.element_uid);
                        }
                    }
                }
            }
            // initialize components outside of locked loop
            for id in comp_ids_to_init {
                lookup_func.lookup_and_setup(id, pipeline)?;
            }

            // initialize all the window elements
            self.initialize_window_elements(pipeline)?;
        }

        // Fetch the compoennt that should be pointing to us
        let src_comp = lookup_func.lookup_and_setup(self.config.src_uid, pipeline)?;

        // link the desired source with the queue and then the queue with the output sink
        self.queue_element.link(&self.output_element)?;
        src_comp.borrow().element().link(&self.queue_element)?;

        // mark setup as complete so as to not rerun
        self.has_setup = true;
        Ok(())
    }

    fn has_setup(&self) -> bool {
        self.has_setup
    }

    // accessor functions
    fn element(&self) -> &Element {
        &self.output_element
    }
    fn uid(&self) -> Uid {
        self.config.uid()
    }

    // Start this component
    fn start_or_run(&self, _pipeline: &gst::Pipeline) -> Result<()> {
        // if we're not main then do nothing
        if !self.is_main {
            return Ok(());
        }

        // if we're main then run the real event loop!
        self.run_event_loop()?;

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
    // only the main window requires the main thread
    fn requires_main(&self) -> bool {
        return self.is_main;
    }
}
