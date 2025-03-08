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

use crate::pipeline;

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
    event_sender: mpsc::Sender<events::RuntimeEvent>,
    event_recver: Arc<Mutex<mpsc::Receiver<events::RuntimeEvent>>>,
    event_thread: Option<thread::JoinHandle<()>>,
}

impl Runtime {
    pub(crate) fn new() -> Result<Runtime> {
        gst::init()?;

        let (send, recv) = mpsc::channel();

        let media_pipeline =
            pipeline::MediaPipeline::new(None, send.clone(), runtime::RuntimeConfig::default())?;

        let runtime = Runtime {
            pipeline: media_pipeline,
            event_recver: Arc::new(Mutex::new(recv)),
            event_sender: send,
            event_thread: None,
        };
        Ok(runtime)
    }

    pub fn run(&mut self) -> Result<()> {
        let event_thread = self.start_background_thread()?;
        self.pipeline.run();

        self.event_sender.send(events::RuntimeEvent::StopThread());
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
