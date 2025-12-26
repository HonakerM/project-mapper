use std::cell::RefCell;
use std::collections::HashSet;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::thread;

use crate::output::WindowConfig;
use crate::output::window::app::WindowAppHandler;
use crate::output::window::config::AvailableWindowConfig;
use anyhow::Result;
use project_mapper_core::runtime_config::shared::Uid;
use project_mapper_runtime::components::shared::{Component, ComponentLookupHelper};
use project_mapper_runtime::gst::prelude::*;
use project_mapper_runtime::types::message::RuntimeMessage;
use winit::event_loop::EventLoop;
use winit::event_loop::EventLoopProxy;

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

// setup global state. While this could be done without the Option
// keep it to allow us to determine which component is the "main" one
// and will start threads/etc
#[derive(Clone, Debug)]
pub(super) struct ProxyWindowState {
    pub event_loop_proxy: Option<WinitPMEventLoopProxy>,
    pub window_comps: HashSet<Uid>,
    pub has_main: bool,
    pub available_config: AvailableWindowConfig,
}
pub(super) static PROXY_WINDOW_STATE: LazyLock<Mutex<Option<ProxyWindowState>>> =
    LazyLock::new(|| Mutex::new(None));

// Struct for components to request a window with
#[derive(Clone, Debug)]
pub struct WindowRequest {
    pub element_name: String,
    pub element_uid: Uid,
    pub config: WindowConfig,
}

#[derive(Clone, Debug)]
pub enum WinitMessage {
    Runtime(RuntimeMessage),
    UpdateWindow(WindowRequest),
    DestroyWindow(WindowRequest),
    AvailableConfig(AvailableWindowConfig),
}

pub type WinitPMEventLoop = EventLoop<WinitMessage>;
pub type WinitPMEventLoopProxy = EventLoopProxy<WinitMessage>;
