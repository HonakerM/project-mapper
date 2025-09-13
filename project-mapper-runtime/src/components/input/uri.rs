use std::sync::{Arc, Mutex, mpsc};

use crate::{
    components::{
        branch::BranchControl,
        shared::{Component, ComponentLookupHelper},
    },
    types::message::RuntimeMessage,
};
use anyhow::{Error, Result, anyhow};
use gst::{Element, element_error, element_warning, glib, prelude::*};
use project_mapper_core::runtime_config::{
    input::{InputComponentConfig, uri::UriConfig},
    shared::{ComponentConfig, Uid},
};

pub struct UriComponent {
    config: InputComponentConfig,
    element: Element,
    branch: BranchControl,
    pipeline: Option<gst::Pipeline>,
}

impl UriComponent {
    fn configure_in_pipeline(&self, pipeline: &gst::Pipeline) -> Result<()> {
        // Add elements to the pipelines and sync status
        pipeline.add(&self.element)?;
        self.element.sync_state_with_parent()?;
        self.branch.add_to_pipeline(pipeline)?;

        // Need to move a new reference into the closure.
        // !!ATTENTION!!:
        // It might seem appealing to use pipeline.clone() here, because that greatly
        // simplifies the code within the callback. What this actually does, however, is creating
        // a memory leak. The clone of a pipeline is a new strong reference on the pipeline.
        // Storing this strong reference of the pipeline within the callback (we are moving it in!),
        // which is in turn stored in another strong reference on the pipeline is creating a
        // reference cycle.
        // DO NOT USE pipeline.clone() TO USE THE PIPELINE WITHIN A CALLBACK
        let pipeline_weak = pipeline.downgrade();

        // Clone sink element so it can be refenced in a callback
        let sink_element = self.branch.get_output()?.clone();

        // Connect to decodebin's pad-added signal, that is emitted whenever
        // it found another stream from the input file and found a way to decode it to its raw format.
        // decodebin automatically adds a src-pad for this raw stream, which
        // we can use to build the follow-up pipeline.
        self.element.connect_pad_added(move |dbin, src_pad| {
            // Here we temporarily retrieve a strong reference on the pipeline from the weak one
            // we moved into this callback.
            let Some(pipeline) = pipeline_weak.upgrade() else {
                return;
            };

            // Try to detect whether the raw stream decodebin provided us with
            // just now is either audio or video (or none of both, e.g. subtitles).
            let (is_audio, is_video) = {
                let media_type = src_pad.current_caps().and_then(|caps| {
                    caps.structure(0).map(|s| {
                        let name = s.name();
                        (name.starts_with("audio/"), name.starts_with("video/"))
                    })
                });

                match media_type {
                    None => {
                        element_warning!(
                            dbin,
                            gst::CoreError::Negotiation,
                            ("Failed to get media type from pad {}", src_pad.name())
                        );

                        return;
                    }
                    Some(media_type) => media_type,
                }
            };

            // We create a closure here, calling it directly below it, because this greatly
            // improves readability for error-handling. Like this, we can simply use the
            // ?-operator within the closure, and handle the actual error down below where
            // we call the insert_sink(..) closure.
            let insert_sink =
                |is_audio, is_video, sink_element: &gst::Element| -> Result<(), Error> {
                    if is_video {
                        // decodebin found a raw videostream, so we build the follow-up pipeline to
                        // display it using the autovideosink.
                        let queue = gst::ElementFactory::make("queue").build()?;
                        let convert = gst::ElementFactory::make("videoconvert").build()?;
                        let scale = gst::ElementFactory::make("videoscale").build()?;

                        let elements = &[&queue, &convert, &scale];
                        pipeline.add_many(elements)?;
                        gst::Element::link_many(elements)?;

                        for e in elements {
                            e.sync_state_with_parent()?
                        }

                        // Get the queue element's sink pad and link the decodebin's newly created
                        // src pad for the video stream to it.
                        let sink_pad = sink_element
                            .static_pad("sink").ok_or(anyhow!("Unable to link to sink pad"))?;
                        src_pad.link(&sink_pad)?;
                    }

                    Ok(())
                };

            // When adding and linking new elements in a callback fails, error information is often sparse.
            // GStreamer's built-in debugging can be hard to link back to the exact position within the code
            // that failed. Since callbacks are called from random threads within the pipeline, it can get hard
            // to get good error information. The macros used in the following can solve that. With the use
            // of those, one can send arbitrary rust types (using the pipeline's bus) into the mainloop.
            // What we send here is unpacked down below, in the iteration-code over sent bus-messages.
            // Because we are using the failure crate for error details here, we even get a backtrace for
            // where the error was constructed. (If RUST_BACKTRACE=1 is set)
            if let Err(err) = insert_sink(is_audio, is_video, &sink_element) {
                // The following sends a message of type Error on the bus, containing our detailed
                // error information.
                element_error!(
                    dbin,
                    gst::LibraryError::Failed,
                    ("Failed to insert sink"),
                    details: gst::Structure::builder("error-details")
                                .field("error", format!("Unable to link Uri sink: {}",err.to_string()))
                                .build()
                );
            }
        });
        Ok(())
    }
}

impl Component for UriComponent {
    // runtime lifecycle functions
    // Construct object
    fn new(unknown_config: &dyn ComponentConfig) -> Result<UriComponent> {
        // parse config and ensure it's correct types
        let config: InputComponentConfig = match unknown_config
            .as_any()
            .downcast_ref::<InputComponentConfig>()
        {
            Some(b) => Ok(b.clone()),
            None => Err(Error::msg(
                "ComponentConfig can not be typed to InputComponentConfig",
            )),
        }?;

        // ensure we have a test config
        let uri_config = match config.config.as_any().downcast_ref::<UriConfig>() {
            Some(b) => Ok(b.clone()),
            None => Err(anyhow!("InputComponentConfig is not UriConfig")),
        }?;

        let element = gst::ElementFactory::make("uridecodebin")
            .name(config.name())
            .property("uri", glib::GString::from(uri_config.uri.clone()))
            .build()
            .map_err(|err| anyhow!("Unable to construct element").context(err))?;

        let comp = Self {
            branch: BranchControl::new(config.name(), false, true)?,
            config: config,
            element: element,
            pipeline: None,
        };
        Ok(comp)
    }

    // Run any post init setup functions
    fn setup(
        &mut self,
        pipeline: &gst::Pipeline,
        _message_sender: mpsc::Sender<RuntimeMessage>,
    ) -> Result<()> {
        self.configure_in_pipeline(pipeline)?;
        self.pipeline = Some(pipeline.clone());
        Ok(())
    }

    fn update(&mut self, config: &dyn ComponentConfig) -> Result<()> {
        // parse config and ensure it's correct types
        let config: InputComponentConfig =
            match config.as_any().downcast_ref::<InputComponentConfig>() {
                Some(b) => Ok(b.clone()),
                None => Err(Error::msg(
                    "ComponentConfig can not be typed to InputComponentConfig",
                )),
            }?;

        // ensure we have a test config
        let uri_config = match config.config.as_any().downcast_ref::<UriConfig>() {
            Some(b) => Ok(b.clone()),
            None => Err(anyhow!("InputComponentConfig is not UriConfig")),
        }?;

        self.element.set_property("uri", uri_config.uri);
        Ok(())
    }

    // accessor functions
    fn output_element(&self) -> Result<&Element> {
        // return the tee element since that's what people should
        // be linking against
        self.branch.get_output()
    }
    fn input_element(&self) -> Result<&Element> {
        Err(anyhow!("Uri component has no input element"))
    }
    fn uid(&self) -> Uid {
        return self.config.uid();
    }
}
