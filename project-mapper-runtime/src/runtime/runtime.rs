//! This example demonstrates how to output GL textures, within an EGL/X11 context provided by the
//! application, and render those textures in the GL application.
//!
//! This example follow common patterns from `glutin`:
//! <https://github.com/rust-windowing/glutin/blob/master/glutin_examples/src/lib.rs>

// {videotestsrc} - { glsinkbin }

use std::sync::{Arc, Mutex};
use std::thread;

use crate::{pipeline, window_handler};

use anyhow::Result;
use std::sync::mpsc;

use project_mapper_core::config::{events, runtime};

pub(crate) struct Runtime {
    pub pipeline: pipeline::MediaPipeline,
    pub event_loop: Option<winit::event_loop::EventLoop<window_handler::Message>>,
    event_sender: mpsc::Sender<events::RuntimeEvent>,
    event_recver: Arc<Mutex<mpsc::Receiver<events::RuntimeEvent>>>,
    event_thread: Option<thread::JoinHandle<()>>,
    window_handler: window_handler::WindowHandler,
}

impl Runtime {
    pub(crate) fn new(config: runtime::RuntimeConfig) -> Result<Runtime> {
        gst::init()?;

        let event_loop: winit::event_loop::EventLoop<window_handler::Message> =
            winit::event_loop::EventLoop::with_user_event().build()?;

        // ControlFlow::Wait pauses the event loop if no events are available to process.
        // This is ideal for non-game applications that only update in response to user
        // input, and uses significantly less power/CPU time than ControlFlow::Poll.
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

        let (send, recv) = mpsc::channel();

        let mut window_handler =
            window_handler::WindowHandler::new(event_loop.create_proxy(), send.clone());

        let media_pipeline =
            pipeline::MediaPipeline::new(config, &mut window_handler, &event_loop, send.clone())?;

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
                    pipeline::MediaPipeline::shutdown_pipeline(pipeline);
                    break;
                }
                events::RuntimeEvent::StopThread() => {
                    println!("Official stop");
                    pipeline::MediaPipeline::shutdown_pipeline(pipeline);
                    break;
                }
            }
        }
    }
}
