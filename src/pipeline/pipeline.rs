//! This example demonstrates how to output GL textures, within an EGL/X11 context provided by the
//! application, and render those textures in the GL application.
//!
//! This example follow common patterns from `glutin`:
//! <https://github.com/rust-windowing/glutin/blob/master/glutin_examples/src/lib.rs>

// {videotestsrc} - { glsinkbin }

use anyhow::Result;
use gst::Element;
use gst_gl::prelude::*;

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
            let id = source_config.id;
            let mut name = id.to_string();

            let mut src_element_option: Option<Element> = None;
            match source_config.source {
                config::source::SourceType::Test {} => {
                    println!("creating test source {id}");
                    name = format!("test-{}", id);
                    src_element_option = Some(source_config.source.create_element(name.clone())?);
                }
            }

            if let Some(src_element) = src_element_option {
                // Add element to pipeline
                pipeline.add(&src_element)?;

                // Add tee to src element to allow multiple linkages
                let tee_name: String = format!("tee-{}", name);
                let src_tee = gst::ElementFactory::make("tee").name(tee_name).build()?;
                pipeline.add(&src_tee)?;

                // Add sync elements before linking
                src_element.sync_state_with_parent()?;
                src_tee.sync_state_with_parent()?;

                // link elements and add mapping for this id to the tee
                src_element.link(&src_tee)?;
                src_elements.insert(id, src_tee.clone());

                // Add elements to list
                elements.push(src_element);
                elements.push(src_tee);
            }
        }

        // construct sinks
        for sink_config in &config.sinks {
            let id = sink_config.id;
            let mut name = id.to_string();

            let mut sink_element_option: Option<Element> = None;

            match &sink_config.sink {
                config::sink::SinkType::OpenGLWindow { monitor } => {
                    name = format!("opengl-{}", id);

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
                        .name(name.clone())
                        .property("sink", &appsink)
                        .build()?;

                    // ensure the window handler knows about this sink
                    window_handler.add_sink(appsink, event_loop, sink_config.clone());

                    sink_element_option = Some(sink);
                }
            }

            // for all sinks add a queue to enable parallel processing
            if let Some(sink) = sink_element_option {
                let queue_name = format!("queue-{}", name);
                let queue_sink = gst::ElementFactory::make("queue")
                    .name(queue_name)
                    .build()?;

                // add to pipeline
                pipeline.add(&queue_sink)?;
                pipeline.add(&sink)?;

                // Add sync elements before linking
                queue_sink.sync_state_with_parent()?;
                sink.sync_state_with_parent()?;

                // link elements and add mapping for this id to the tee
                queue_sink.link(&sink)?;
                sink_elements.insert(id, queue_sink.clone());

                // add both to elements
                elements.push(sink);
                elements.push(queue_sink);
            }
        }

        // ensure they're all added and configured to the pipeline before linking
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
