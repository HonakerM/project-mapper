//! This example demonstrates how to output GL textures, within an EGL/X11 context provided by the
//! application, and render those textures in the GL application.
//!
//! This example follow common patterns from `glutin`:
//! <https://github.com/rust-windowing/glutin/blob/master/glutin_examples/src/lib.rs>

// {videotestsrc} - { glsinkbin }

use std::{
    ffi::{CStr, CString},
    mem,
    num::NonZeroU32,
    ptr,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use anyhow::{Context, Result};
use glib::{BoolError, clone::Downgrade};
use glutin::{
    config::GetGlConfig as _,
    context::AsRawContext as _,
    display::{AsRawDisplay as _, GetGlDisplay as _},
    prelude::*,
};
use glutin_winit::GlWindow as _;
use gst::{Element, PadProbeReturn, PadProbeType, QueryViewMut, element_error};
use gst_gl::prelude::*;
use raw_window_handle::HasWindowHandle as _;
use winit::{
    dpi::{LogicalSize, PhysicalPosition, PhysicalSize},
    window,
};

use crate::window_handler;
use crate::{config, window_handler::WindowHandler};

use std::sync::mpsc;

use crate::config::events;
use crate::config::runtime;
use std::collections::HashMap;

pub(crate) struct MediaPipeline {
    pub pipeline: gst::Pipeline,
    runtime_sender: mpsc::Sender<events::RuntimeEvent>,
    pub config: runtime::RuntimeConfig,
    elements: Vec<gst::Element>,
}

impl MediaPipeline {
    pub(crate) fn new(
        window_handler: &mut WindowHandler,
        runtime_sender: mpsc::Sender<events::RuntimeEvent>,
        config: runtime::RuntimeConfig,
    ) -> Result<MediaPipeline> {
        gst::init()?;

        let (elements, pipeline) = MediaPipeline::create_pipeline(&config, window_handler)?;

        let pipeline: gst::Pipeline = pipeline.to_owned();

        let media_pipeline: MediaPipeline = MediaPipeline {
            pipeline: pipeline,
            runtime_sender: runtime_sender,
            config: config,
            elements: elements,
        };

        Ok(media_pipeline)
    }

    pub(crate) fn start(&mut self) {
        self.pipeline.set_state(gst::State::Playing).unwrap();
    }

    pub fn shutdown_pipeline(pipeline: gst::Pipeline) {
        pipeline.send_event(gst::event::Eos::new());
        pipeline.set_state(gst::State::Null).unwrap();
    }

    fn create_pipeline(
        config: &runtime::RuntimeConfig,
        window_handler: &mut window_handler::WindowHandler,
    ) -> Result<(Vec<gst::Element>, gst::Pipeline)> {
        let pipeline = gst::Pipeline::default();

        let mut elements = Vec::<gst::Element>::new();
        let source_config = config.sources.get(0).unwrap();

        let mut src_elements = HashMap::new();
        match source_config.source {
            config::source::SourceType::Test {} => {
                println!("creating test source");
                let var = source_config.source.create_element()?;
                src_elements.insert(source_config.id, var.clone());

                elements.push(var);
            }
        }

        let src: &Element = src_elements[&source_config.id].as_ref();

        let caps = gst_video::VideoCapsBuilder::new()
            .features([gst_gl::CAPS_FEATURE_MEMORY_GL_MEMORY])
            .format(gst_video::VideoFormat::Rgba)
            .field("texture-target", "2D")
            .build();

        let appsink = gst_app::AppSink::builder()
            .enable_last_sample(true)
            .max_buffers(1)
            .caps(&caps)
            .build();

        let sink = gst::ElementFactory::make("glsinkbin")
            .name("gl-sink-1")
            .property("sink", &appsink)
            .build()?;

        elements.push(sink.clone());
        pipeline.add_many(&elements)?;

        for e in &elements {
            e.sync_state_with_parent()?
        }

        src.link(&sink)?;
        let sink_config = config.sinks.get(0).unwrap();

        window_handler.add_sink(appsink, sink_config.clone());

        Ok((elements, pipeline))
    }
}
