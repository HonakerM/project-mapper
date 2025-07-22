use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::mpsc;
use std::thread;

use crate::components::branch::BranchControl;
use crate::components::runtime::DefaultRuntimeComponent;
use crate::components::shared::{Component, ComponentLookupHelper};
use crate::types::message::RuntimeMessage;
use crate::utils::winit::WinitPMEventLoop;
use crate::utils::winit::WinitPMEventLoopProxy;
use crate::utils::winit::get_monitor_by_name;
use crate::utils::winit::get_video_mode_for_config;
use anyhow::Context;
use anyhow::Ok;
use anyhow::anyhow;
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
use winit::event_loop;
use winit::event_loop::EventLoopBuilder;
use winit::platform::pump_events::EventLoopExtPumpEvents;
use winit::window::Window;
use winit::window::WindowBuilder;

// helper struct to store information about winit. This
// will only be held by the main component
struct WinitState {
    message_sender_thread: Option<thread::JoinHandle<Result<()>>>,
    event_loop: Option<WinitPMEventLoop>,
    // needed to keep reference to a window
    windows: HashMap<Uid, Window>,
}
impl Default for WinitState {
    fn default() -> Self {
        Self {
            message_sender_thread: None,
            event_loop: None,
            windows: HashMap::new(),
        }
    }
}

// use thread local window state since we only ever need it on the main thread
thread_local! {
    static GLOBAL_WINDOW_STATE: RefCell<WinitState> = RefCell::new(WinitState::default());
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
    pub event_loop_proxy: Option<WinitPMEventLoopProxy>,
    pub window_configs: HashMap<String, WindowRequest>,
}
static PROXY_WINDOW_STATE: LazyLock<Mutex<Option<ProxyWindowState>>> =
    LazyLock::new(|| Mutex::new(None));

pub struct WindowComponent {
    config: OutputComponentConfig,
    window_config: WindowConfig,

    // message sender for runtime events
    message_sender: Option<mpsc::Sender<RuntimeMessage>>,

    // gst elements
    branch: BranchControl,
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
        let event_loop: WinitPMEventLoop = EventLoopBuilder::with_user_event().build()?;
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

        GLOBAL_WINDOW_STATE.replace(WinitState {
            // don't create the message sender until we have it in run
            message_sender_thread: None,
            event_loop: Some(event_loop),
            windows: windows,
        });

        Ok(())
    }

    fn configure_window(
        event_loop: &WinitPMEventLoop,
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
                        proxy.send_event(event.clone())?;
                        if event == RuntimeMessage::ExitRuntime() {
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
    ) -> Result<RuntimeMessage> {
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

        let mut exit_event: Option<RuntimeMessage> = None;
        let mut exit_error: Option<Error> = None;

        while exit_event.is_none() {
            event_loop.pump_events(None, |event, _event_loop_target| {
                match event {
                    Event::UserEvent(user_event) => {
                        exit_event = Some(user_event);
                    }
                    Event::WindowEvent {
                        event: window_event,
                        ..
                    } => match window_event {
                        WindowEvent::CloseRequested => {
                            // Send a message to stop the event loop
                            let send_result = message_sender.send(RuntimeMessage::ExitRuntime());
                            if let Err(err) = send_result {
                                exit_error = Some(err.into());
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
            });
        }

        // after running replace the global state and event loop to retain references to the windows
        winit_state.event_loop = Some(event_loop);
        GLOBAL_WINDOW_STATE.set(winit_state);

        // if there was an exit error return it first
        if let Some(err) = exit_error {
            return Err(err);
        }

        // get the exit event if it was created
        if let Some(event) = exit_event {
            Ok(event)
        } else {
            Err(anyhow!(
                "Event loop exited without event. Should never happen due to loop conditions"
            ))
        }
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
        match config.config {
            OutputConfig::Window(window_config) => {
                let window_config = window_config.clone();

                let branch = BranchControl::new(local_config.name(), true, false)?;
                let output_element = gst::ElementFactory::make("glimagesink")
                    .name(local_config.name())
                    .build()?;
                output_element.set_property("sync", &false);
                let mut window_component = Self {
                    config: local_config,
                    window_config: window_config,

                    branch: branch,
                    output_element: output_element,
                    has_setup: false,

                    message_sender: None,

                    // default to not main. This will be configured during initialize_global_state
                    is_main: false,
                };
                window_component.initialize_global_state()?;
                Ok(window_component)
            }
            _ => Err(Error::msg(
                "OutputConfig is not correct type for this component",
            )),
        }
    }

    // Run any post init setup functions
    fn setup(
        &mut self,
        pipeline: &gst::Pipeline,
        message_sender: mpsc::Sender<RuntimeMessage>,
        lookup_func: &dyn ComponentLookupHelper,
    ) -> Result<()> {
        if self.has_setup {
            return Ok(());
        }

        // copy the message sender into the object
        self.message_sender = Some(message_sender.clone());

        // Add both elements to the pipelines and sync status
        pipeline.add(&self.output_element)?;
        self.output_element.sync_state_with_parent()?;

        // construct branch and wrap
        self.branch.add_to_pipeline(pipeline)?;
        self.branch.link_wrapped(&self.output_element)?;

        // If we're the main function than recursively call setup on all available window references
        // this ensures all pipeline elements that require a window have been added to the pipeline
        // ! Note this must come after adding the components to the pipeline:

        if self.is_main {
            // only lock global state while gathering components to initialize
            let mut comp_ids_to_init: Vec<Uid> = vec![];
            {
                let proxy_state = PROXY_WINDOW_STATE.lock().or(Err(Error::msg(
                    "Unable to aquire window proxy lock. Should not happen in normal operation",
                )))?;
                if let Some(state) = proxy_state.as_ref() {
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
                lookup_func.lookup_and_setup(id, pipeline, message_sender.clone())?;
            }

            // initialize all the window elements
            self.initialize_window_elements(pipeline)?;
        }

        // Fetch the compoennt that should be pointing to us
        let src_comp =
            lookup_func.lookup_and_setup(self.config.src_uid, pipeline, message_sender.clone())?;

        // link the desired source with the branch
        src_comp
            .borrow()
            .element()?
            .link(self.branch.get_input()?)?;

        // mark setup as complete so as to not rerun
        self.has_setup = true;
        Ok(())
    }

    fn has_setup(&self) -> bool {
        self.has_setup
    }

    // accessor functions
    fn element(&self) -> Result<&Element> {
        Ok(&self.output_element)
    }
    fn uid(&self) -> Uid {
        self.config.uid()
    }

    // Run this component
    fn run(
        &self,
        _pipeline: &gst::Pipeline,
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
        self.run_event_loop(message_sender.clone(), message_broker.clone())
        /*
        // if we exited the event loop we also must have exited the event watcher...
         */
    }

    // Stop this component
    fn destroy(&mut self) -> Result<()> {
        // if we're not main then do nothing
        if !self.is_main {
            return Ok(());
        }

        // else destroy/drop all windows
        let mut winit_state = GLOBAL_WINDOW_STATE.take();
        winit_state.windows.clear();

        // exit the event loop
        if let Some(event_loop) = winit_state.event_loop {
            event_loop.exit();
        }

        // ensure we destory/join the event listener thread
        if let Some(event_handle) = winit_state.message_sender_thread {
            let possible_error = if event_handle.is_finished() {
                event_handle.join().map_err(|panic_err| {
                    // Try to extract a meaningful panic message
                    if let Some(s) = panic_err.downcast_ref::<&'static str>() {
                        anyhow!("Thread panicked: {}", s)
                    } else if let Some(s) = panic_err.downcast_ref::<String>() {
                        anyhow!("Thread panicked: {}", s)
                    } else {
                        anyhow!("Thread panicked with non-string payload")
                    }
                })
            } else {
                Err(anyhow!(
                    "Event watcher thread not finished even though event is done"
                ))
            }?;
            possible_error?;
        }

        Ok(())
    }
    // only the main window requires the main thread
    fn requires_main(&self) -> bool {
        return self.is_main;
    }
}
