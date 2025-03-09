//! This example demonstrates how to output GL textures, within an EGL/X11 context provided by the
//! application, and render those textures in the GL application.
//!
//! This example follow common patterns from `glutin`:
//! <https://github.com/rust-windowing/glutin/blob/master/glutin_examples/src/lib.rs>

// {videotestsrc} - { glsinkbin }

use std::thread;
use std::{
    ffi::{CStr, CString},
    mem,
    num::NonZeroU32,
    ptr,
    sync::{Arc, Mutex},
};

use crate::{pipeline, window_handler};

use anyhow::{Context, Result};
use glutin::{
    config::GetGlConfig as _,
    context::AsRawContext as _,
    display::{AsRawDisplay as _, GetGlDisplay as _},
    prelude::*,
};
use glutin_winit::GlWindow as _;
use gst::{PadProbeReturn, PadProbeType, QueryViewMut, element_error};
use gst_gl::prelude::*;
use raw_window_handle::HasWindowHandle as _;
use std::sync::mpsc;
use winit::dpi::{LogicalSize, PhysicalPosition, PhysicalSize};

use crate::config::{events, runtime};

#[derive(Debug)]
pub(crate) enum Message {
    Frame(gst_video::VideoInfo, gst::Buffer),
    BusMessage(gst::Message),
}

pub(crate) struct Runtime {
    pub pipeline: pipeline::MediaPipeline,
    pub event_loop: Option<winit::event_loop::EventLoop<window_handler::Message>>,
    event_sender: mpsc::Sender<events::RuntimeEvent>,
    event_recver: Arc<Mutex<mpsc::Receiver<events::RuntimeEvent>>>,
    event_thread: Option<thread::JoinHandle<()>>,
    window_handler: window_handler::WindowHandler,
}

impl Runtime {
    pub(crate) fn new() -> Result<Runtime> {
        gst::init()?;

        let event_loop: winit::event_loop::EventLoop<window_handler::Message> =
            winit::event_loop::EventLoop::with_user_event().build()?;

        // ControlFlow::Wait pauses the event loop if no events are available to process.
        // This is ideal for non-game applications that only update in response to user
        // input, and uses significantly less power/CPU time than ControlFlow::Poll.
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

        // ControlFlow::Wait pauses the event loop if no events are available to process.
        // This is ideal for non-game applications that only update in response to user
        // input, and uses significantly less power/CPU time than ControlFlow::Poll.
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

        let (send, recv) = mpsc::channel();

        let mut window_handler =
            window_handler::WindowHandler::new(event_loop.create_proxy(), send.clone());

        let media_pipeline = pipeline::MediaPipeline::new(
            &mut window_handler,
            None,
            send.clone(),
            runtime::RuntimeConfig::default(),
        )?;

        let runtime = Runtime {
            pipeline: media_pipeline,
            event_recver: Arc::new(Mutex::new(recv)),
            event_sender: send,
            event_thread: None,
            event_loop: Some(event_loop),
            window_handler: window_handler,
        };
        Ok(runtime)
    }

    pub fn run(&mut self) -> Result<()> {
        let event_thread = self.start_background_thread()?;

        self.pipeline.start();
        let event_loop = std::mem::replace(&mut self.event_loop, None).expect("uh oh");
        event_loop.run_app(&mut self.window_handler);

        event_thread.join();
        Ok(())
    }

    fn start_background_thread(&self) -> Result<thread::JoinHandle<()>> {
        let recv: Arc<Mutex<mpsc::Receiver<events::RuntimeEvent>>> = self.event_recver.clone();
        let pipeline = self.pipeline.pipeline.clone();

        let background_thread = thread::spawn(move || {
            Runtime::listen_for_events(recv, pipeline);
        });
        return Ok(background_thread);
    }

    fn listen_for_events(
        event_recver: Arc<Mutex<mpsc::Receiver<events::RuntimeEvent>>>,
        pipeline: gst::Pipeline,
    ) {
        for event in event_recver.lock().unwrap().iter() {
            match event {
                events::RuntimeEvent::UserExit() => {
                    println!("User stop");
                    pipeline::MediaPipeline::shutdown_pipeline(pipeline.clone())
                }
                events::RuntimeEvent::StopThread() => {
                    println!("Official stop");
                    pipeline::MediaPipeline::shutdown_pipeline(pipeline);
                    break;
                }
                _ => println!("Unknown event"),
            }
        }
    }
}
