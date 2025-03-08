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
use gst::{PadProbeReturn, PadProbeType, QueryViewMut, element_error};
use gst_gl::prelude::*;
use raw_window_handle::HasWindowHandle as _;
use winit::dpi::{LogicalSize, PhysicalPosition, PhysicalSize};

use crate::{config, opengl};

use std::sync::mpsc;

use crate::config::events;
use crate::config::runtime;
use std::collections::HashMap;

pub(crate) struct MediaPipeline {
    pub pipeline: gst::Pipeline,
    runtime_sender: mpsc::Sender<events::RuntimeEvent>,
    pub app: opengl::OpenGLApp,
    pub config: runtime::RuntimeConfig,
    elements: Vec<gst::Element>,
}

impl MediaPipeline {
    pub(crate) fn new(
        gl_element: Option<&gst::Element>,
        runtime_sender: mpsc::Sender<events::RuntimeEvent>,
        config: runtime::RuntimeConfig,
    ) -> Result<MediaPipeline> {
        gst::init()?;

        let (elements, pipeline, appsink) = MediaPipeline::create_pipeline(gl_element, &config)?;

        let pipeline: gst::Pipeline = pipeline.to_owned();
        let app = opengl::OpenGLApp::new(None, runtime_sender.clone(), appsink)?;

        let media_pipeline: MediaPipeline = MediaPipeline {
            pipeline: pipeline,
            app: app,
            runtime_sender: runtime_sender,
            config: config,
            elements: elements,
        };

        Ok(media_pipeline)
    }

    pub(crate) fn run(&mut self) {
        self.app.setup(&self.pipeline);
        self.pipeline.set_state(gst::State::Playing).unwrap();
        self.app.run();
    }

    pub fn shutdown_pipeline(pipeline: gst::Pipeline) {
        pipeline.send_event(gst::event::Eos::new());
        pipeline.set_state(gst::State::Null).unwrap();
    }

    fn create_pipeline(
        gl_element: Option<&gst::Element>,
        config: &runtime::RuntimeConfig,
    ) -> Result<(Vec<gst::Element>, gst::Pipeline, gst_app::AppSink)> {
        let pipeline = gst::Pipeline::default();

        let mut elements = Vec::<gst::Element>::new();
        let source_config = config.sources.get(0).unwrap();

        let mut src_elements = HashMap::new();
        match source_config.source {
            config::source::SourceType::Test {} => {
                let var = source_config.source.create_element()?;
                src_elements.insert(source_config.id, var.clone());

                elements.push(var);
            }
        }

        let src = src_elements[&source_config.id].as_ref();

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

        if let Some(gl_element) = gl_element {
            let glupload = gst::ElementFactory::make("glupload").build()?;

            pipeline.add_many([&src, &glupload])?;
            pipeline.add(gl_element)?;
            pipeline.add(&appsink)?;

            src.link(&glupload)?;
            glupload.link(gl_element)?;
            gl_element.link(&appsink)?;

            Ok((elements, pipeline, appsink))
        } else {
            let sink = gst::ElementFactory::make("glsinkbin")
                .property("sink", &appsink)
                .build()?;

            pipeline.add_many([&src, &sink])?;
            src.link(&sink)?;

            Ok((elements, pipeline, appsink))
        }
    }
}
