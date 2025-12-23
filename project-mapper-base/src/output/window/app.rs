use std::collections::HashMap;
use std::sync::mpsc;

use gst::Pipeline;
use gst::glib::object::Cast;
use gst::prelude::GstBinExt;
use gst_video::prelude::VideoOverlayExtManual;
use log::info;
use project_mapper_core::runtime_config::shared::Uid;
use project_mapper_runtime::gst;
use project_mapper_runtime::gst_video;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};

use crate::output::WindowConfig;
use crate::output::window::config::AvailableWindowConfig;
use crate::output::window::config::WindowMode;
use crate::output::window::state::WinitPMEventLoopProxy;
use crate::output::window::state::{WindowRequest, WinitMessage};
use crate::output::window::utils::get_monitor_by_name;
use crate::output::window::utils::get_video_mode_for_config;
use anyhow::{Context as _, Error, Result};
use project_mapper_runtime::types::message::RuntimeMessage;

#[derive(Debug)]
pub(super) struct WindowAppHandler {
    // need to know about the pipeline for linking
    pipeline: Pipeline,
    message_sender: mpsc::Sender<RuntimeMessage>,

    loopback_proxy: WinitPMEventLoopProxy,

    pub last_events: Vec<WinitMessage>,
    pub exit_err: Option<Error>,

    // needed to keep reference to a window
    windows: HashMap<Uid, Window>,
    window_lookup: HashMap<WindowId, Uid>,
}

impl WindowAppHandler {
    pub fn destory_window(&mut self, uid: &Uid) {
        if let Some(window) = self.windows.remove(uid) {
            self.window_lookup.remove(&window.id());
        }
    }
    pub fn destory(&mut self) {
        self.windows.clear();
        self.window_lookup.clear();
    }
    pub fn new(
        pipeline: Pipeline,
        message_sender: mpsc::Sender<RuntimeMessage>,
        loopback_proxy: WinitPMEventLoopProxy,
    ) -> Self {
        return Self {
            pipeline: pipeline,
            message_sender: message_sender,
            loopback_proxy: loopback_proxy,
            last_events: vec![],
            exit_err: None,
            windows: HashMap::new(),
            window_lookup: HashMap::new(),
        };
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

    pub fn has_event(&mut self) -> bool {
        !self.last_events.is_empty()
    }
    pub fn get_next_event(&mut self) -> Option<WinitMessage> {
        self.last_events.pop()
    }

    pub fn has_window(&self, id: &Uid) -> bool {
        self.windows.contains_key(id)
    }

    pub fn process_window_request(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_request: WindowRequest,
    ) {
        // skip windows we've already created
        if let Some(window) = self.windows.get(&window_request.element_uid) {
            WindowAppHandler::configure_window(event_loop, window, window_request.config);
            return;
        }

        for monitor_handle in event_loop.available_monitors() {
            println!("We have monitor {:?}", monitor_handle.name());
        }
        let window = event_loop
            .create_window(
                WindowAttributes::default()
                    .with_title(window_request.element_name.clone())
                    .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0)),
            )
            .unwrap();

        // get the gst element as a video overlay
        let element = self
            .pipeline
            .by_name(&window_request.element_name)
            .with_context(|| {
                format!(
                    "Element with name '{}' not found in pipeline",
                    window_request.element_name
                )
            })
            .unwrap();
        let overlay = element
            .dynamic_cast::<gst_video::VideoOverlay>()
            .map_err(|_| anyhow::anyhow!("Failed to cast element to VideoOverlay"))
            .unwrap();

        // Obtain the raw window handle from winit
        let raw_handle = window.window_handle().unwrap().as_raw();

        // Extract platform-specific handle ID
        let handle_id = match raw_handle {
            RawWindowHandle::Xlib(h) => h.window as usize,
            RawWindowHandle::Wayland(h) => h.surface.as_ptr() as usize,
            RawWindowHandle::Win32(h) => h.hwnd.get() as usize,
            RawWindowHandle::WinRt(h) => h.core_window.as_ptr() as usize,
            RawWindowHandle::AppKit(h) => h.ns_view.as_ptr() as usize,
            _ => panic!("Unsupported platform: cannot get raw window handle"),
        };

        // set the gstreamer element to output to this handle!
        unsafe {
            overlay.set_window_handle(handle_id);
        }

        // configure the window based on the config
        WindowAppHandler::configure_window(&event_loop, &window, window_request.config.clone())
            .unwrap();

        window.request_redraw();

        // hold onto window ref
        self.window_lookup
            .insert(window.id(), window_request.element_uid);
        self.windows.insert(window_request.element_uid, window);
        info!(
            "Created window for output element {}",
            window_request.element_uid
        );
    }
}

impl ApplicationHandler<WinitMessage> for WindowAppHandler {
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: winit::event::StartCause) {
        match cause {
            winit::event::StartCause::Init => {
                self.loopback_proxy
                    .send_event(WinitMessage::AvailableConfig(
                        AvailableWindowConfig::from_monitor_handles(
                            event_loop.available_monitors(),
                        ),
                    ))
                    .unwrap();
            }
            _ => {}
        }
    }

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {}

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
            other_event => {
                //                    info!("Recieved event: {:?}", other_event);
            }
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: WinitMessage) {
        match event {
            WinitMessage::UpdateWindow(request) => {
                self.process_window_request(event_loop, request);
            }
            msg => {
                self.last_events.push(msg);
            }
        }
    }
}
