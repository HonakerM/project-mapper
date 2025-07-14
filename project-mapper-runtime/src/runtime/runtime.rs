use anyhow::{Error, Result};
use gst::MessageView;
use gst::{glib, prelude::*};
use project_mapper_core::runtime_config::RuntimeConfig;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{EventLoop, EventLoopBuilder, EventLoopWindowTarget},
};

use crate::types::message::RuntimeMessage;

pub struct Runtime {
    pub config: RuntimeConfig,
    pub event_loop: EventLoop<RuntimeMessage>,

    exit_error: Option<Error>,
}

impl Runtime {
    pub fn new(config: RuntimeConfig) -> Result<Self> {
        // Create the event loop with a runtime message as the user event
        let event_loop: EventLoop<RuntimeMessage> = EventLoopBuilder::with_user_event().build()?;

        // ControlFlow::Wait pauses the event loop if no events are available to process.
        // This is ideal for non-game applications that only update in response to user
        // input, and uses significantly less power/CPU time than ControlFlow::Poll.
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

        Ok(Self {
            config: config,
            event_loop: event_loop,
            exit_error: None,
        })
    }

    pub fn run(self) -> Result<()> {
        self.event_loop.run(move |event, loop_ref| {
            Runtime::event_handler(event, loop_ref);
        })?;

        Ok(())
    }

    fn event_handler(
        event: Event<RuntimeMessage>,
        loop_ref: &EventLoopWindowTarget<RuntimeMessage>,
    ) {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    // Stop the pipeline
                    loop_ref.exit();
                }
                WindowEvent::Resized(_new_size) => {
                    // Optionally handle resize
                }
                _ => (),
            },
            Event::UserEvent(message) => match message {
                RuntimeMessage::UserExit() => {
                    loop_ref.exit();
                }
                RuntimeMessage::GSTMessage(gst_message) => {
                    if let MessageView::Error(err) = gst_message.view() {
                        let src = gst_message
                            .src()
                            .map(|s| s.path_string())
                            .unwrap_or_else(|| glib::GString::from("UNKNOWN"));
                        let error = err.error();
                        let debug = err.debug();
                        ("Received error from {src}: {error} (debug: {debug:?})");
                    }
                }
            },
            _ => (),
        }
    }
}
