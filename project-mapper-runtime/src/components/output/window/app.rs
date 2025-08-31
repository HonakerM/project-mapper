use std::collections::HashMap;
use std::sync::mpsc;

use gst::glib::object::Cast;
use gst::prelude::GstBinExt;
use gst::Pipeline;
use gst_video::prelude::VideoOverlayExtManual;
use log::info;
use project_mapper_core::runtime_config::output::window::{WindowConfig, WindowMode};
use project_mapper_core::runtime_config::shared::Uid;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};
use winit::application::ApplicationHandler;

use crate::components::output::window::state::PROXY_WINDOW_STATE;
use crate::types::message::RuntimeMessage;
use crate::utils::winit::{get_monitor_by_name, get_video_mode_for_config, WinitPMEventLoop};
use anyhow::{Context as _, Error, Result};




pub(super) struct WindowAppHandler {
    // need to know about the pipeline for linking
    pipeline: Pipeline,
    message_sender: mpsc::Sender<RuntimeMessage>,

    pub last_event: Option<RuntimeMessage>,
    pub exit_err: Option<Error>,
    
    // needed to keep reference to a window
    windows: HashMap<Uid, Window>,
}

impl WindowAppHandler {
    pub fn destory(&mut self)  {
        self.windows.clear();
    }
    pub fn new(pipeline: Pipeline, message_sender: mpsc::Sender<RuntimeMessage>) -> Self {
        return Self { pipeline: pipeline, message_sender: message_sender, last_event: None, exit_err: None, windows: HashMap::new() }
    }

    fn configure_window(
        event_loop: &ActiveEventLoop,
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
}

impl ApplicationHandler<RuntimeMessage> for WindowAppHandler {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        // create all the required windows and update the global state
        let mut global_state = PROXY_WINDOW_STATE.lock().expect(
            "Unable to aquire window proxy lock. Should not happen in normal operation",
        );
        if let Some(state) = global_state.as_mut() {
            // while we have the lock setup all windows
            for window_request in state.window_configs.values() {
                // skip windows we've already created
                if self.windows.contains_key(&window_request.element_uid)  {
                    continue;
                }

                let window = event_loop.create_window(
                    WindowAttributes::default()
                    .with_title(window_request.element_name.clone())
                    .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
                ).unwrap();


                // get the gst element as a video overlay
                let element = self.pipeline
                    .by_name(&window_request.element_name)
                    .with_context(|| {
                        format!(
                            "Element with name '{}' not found in pipeline",
                            window_request.element_name
                        )
                    }).unwrap();
                let overlay = element
                    .dynamic_cast::<gst_video::VideoOverlay>()
                    .map_err(|_| anyhow::anyhow!("Failed to cast element to VideoOverlay")).unwrap();

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
                WindowAppHandler::configure_window(
                    &event_loop,
                    &window,
                    window_request.config.clone(),
                ).unwrap();

                // hold onto window ref
                self.windows.insert(window_request.element_uid, window);

                info!("Created window for output element {}", window_request.element_uid)
            }
        }
    
    }

    fn window_event(
            &mut self,
            event_loop: &winit::event_loop::ActiveEventLoop,
            window_id: winit::window::WindowId,
            event: winit::event::WindowEvent,
        ) {
             match event {
                WindowEvent::CloseRequested => {
                    // Send a message to stop the event loop
                    let send_result = self.message_sender.send(RuntimeMessage::ExitRuntime());
                    if let Err(err) = send_result {
                        self.exit_err = Some(err.into());
                    }
                }
                _ => {}
            }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: RuntimeMessage) {
        self.last_event = Some(event);

    }
}
