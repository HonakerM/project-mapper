use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::components::branch::BranchControl;
use crate::components::output::window::app::WindowAppHandler;
use crate::components::runtime::DefaultRuntimeComponent;
use crate::components::shared::{Component, ComponentLookupHelper};
use crate::types::message::RuntimeMessage;
use crate::utils::winit::get_monitor_by_name;
use crate::utils::winit::get_video_mode_for_config;
use anyhow::Context;
use anyhow::Ok;
use anyhow::anyhow;
use anyhow::{Error, Result};
use gst::Element;
use gst::prelude::*;
use gst_video::prelude::*;
use log::{debug, info};
use project_mapper_core::runtime_config::output::window::WindowMode;
use project_mapper_core::runtime_config::shared::Uid;
use project_mapper_core::runtime_config::{
    output::{OutputComponentConfig, window::WindowConfig},
    shared::ComponentConfig,
};
use raw_window_handle::HasWindowHandle;
use raw_window_handle::RawWindowHandle;
use winit::event::Event;
use winit::event::WindowEvent;
use winit::event_loop;
use winit::event_loop::EventLoop;
use winit::event_loop::EventLoopBuilder;
use winit::event_loop::EventLoopProxy;
use winit::platform::pump_events::EventLoopExtPumpEvents;
use winit::window::Window;

// helper struct to store information about winit. This
// will only be held by the main component
#[derive(Debug)]
pub(super) struct WinitState {
    pub message_sender_thread: Option<thread::JoinHandle<Result<()>>>,
    pub event_loop: Option<WinitPMEventLoop>,
    pub handler: Option<WindowAppHandler>,
}
impl Default for WinitState {
    fn default() -> Self {
        Self {
            message_sender_thread: None,
            event_loop: None,
            handler: None,
        }
    }
}

// use thread local window state since we only ever need it on the main thread
thread_local! {
    pub(super) static GLOBAL_WINDOW_STATE: RefCell<WinitState> = RefCell::new(WinitState::default());
}

// Struct for components to request a window with
#[derive(Clone, Debug)]
pub(super) struct WindowRequest {
    pub element_name: String,
    pub element_uid: Uid,
    pub config: WindowConfig,
}

#[derive(Clone)]
pub enum WinitMessage {
    Runtime(RuntimeMessage),
    UpdateWindow(WindowRequest),
}

pub type WinitPMEventLoop = EventLoop<WinitMessage>;
pub type WinitPMEventLoopProxy = EventLoopProxy<WinitMessage>;

// setup global state. While this could be done without the Option
// keep it to allow us to determine which component is the "main" one
// and will start threads/etc
#[derive(Clone)]
pub(super) struct ProxyWindowState {
    pub event_loop_proxy: Option<WinitPMEventLoopProxy>,
}
pub(super) static PROXY_WINDOW_STATE: LazyLock<Mutex<Option<ProxyWindowState>>> =
    LazyLock::new(|| Mutex::new(None));
