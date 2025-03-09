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
        config: runtime::RuntimeConfig,
        window_handler: &mut WindowHandler,
        event_loop: &winit::event_loop::EventLoop<window_handler::Message>,
        runtime_sender: mpsc::Sender<events::RuntimeEvent>,
    ) -> Result<MediaPipeline> {
        gst::init()?;

        let (elements, pipeline) =
            MediaPipeline::create_pipeline(&config, window_handler, event_loop)?;

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
        event_loop: &winit::event_loop::EventLoop<window_handler::Message>,
    ) -> Result<(Vec<gst::Element>, gst::Pipeline)> {
        let pipeline = gst::Pipeline::default();

        let mut elements: Vec<gst::Element> = Vec::<gst::Element>::new();
        let mut src_elements: HashMap<u32, Element> = HashMap::new();
        let mut sink_elements: HashMap<u32, Element> = HashMap::new();

        // construct sources
        for source_config in &config.sources {
            match source_config.source {
                config::source::SourceType::Test {} => {
                    let id = source_config.id;
                    let name: String = format!("test-{}", id);
                    println!("creating test source {id}");
                    let var = source_config.source.create_element(name)?;
                    src_elements.insert(id, var.clone());

                    elements.push(var);
                }
            }
        }

        // construct sinks
        for sink_config in &config.sinks {
            match &sink_config.sink {
                config::sink::SinkType::OpenGLWindow { monitor } => {
                    let id = sink_config.id;
                    let name: String = format!("opengl-{}", id);
                    println!("creating opengl window sink {name}");

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
                        .name(name)
                        .property("sink", &appsink)
                        .build()?;

                    window_handler.add_sink(appsink, event_loop, sink_config.clone());

                    sink_elements.insert(id, sink.clone());
                    elements.push(sink);
                }
            }
        }

        // ensure they're all added and configured to the pipeline before linking
        pipeline.add_many(&elements)?;

        for e in &elements {
            e.sync_state_with_parent()?
        }

        // tie the regions together
        for region in &config.regions {
            match region.region {
                config::runtime::RegionType::Display { source, sink } => {
                    let src: &Element = src_elements[&source].as_ref();
                    let sink: &Element = sink_elements[&sink].as_ref();

                    src.link(sink)?;
                }
            }
        }

        Ok((elements, pipeline))
    }
}
