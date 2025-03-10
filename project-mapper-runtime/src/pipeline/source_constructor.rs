use anyhow::Error;
use glib::clone::Downgrade;
use gst::{
    Element, element_error, element_warning,
    prelude::{ElementExt, ElementExtManual, GstBinExtManual, GstObjectExt, PadExt},
};
use project_mapper_core::config::source::{self, SourceType};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, glib::Boxed)]
#[boxed_type(name = "ErrorValue")]
struct ErrorValue(Arc<Mutex<Option<Error>>>);

pub trait SourceTypeConstructor {
    fn create_element(&self, id: String) -> Result<Element, glib::BoolError>;
    fn initialize_element(
        &self,
        src_element: &gst::Element,
        sink_element: &gst::Element,
        pipeline: &gst::Pipeline,
    ) -> Result<(), glib::error::BoolError>;
}

impl SourceTypeConstructor for &source::Test {
    fn create_element(&self, id: String) -> Result<Element, glib::BoolError> {
        let name = format!("test-{}", id);
        gst::ElementFactory::make("videotestsrc").name(name).build()
    }

    fn initialize_element(
        &self,
        src_element: &gst::Element,
        sink_element: &gst::Element,
        pipeline: &gst::Pipeline,
    ) -> Result<(), glib::error::BoolError> {
        src_element.link(sink_element)
    }
}

impl SourceTypeConstructor for &source::URI {
    fn create_element(&self, id: String) -> Result<Element, glib::BoolError> {
        let name = format!("uri-{}", id);
        gst::ElementFactory::make("uridecodebin")
            .name(name)
            .property("uri", glib::GString::from(self.uri.clone()))
            .build()
    }

    fn initialize_element(
        &self,
        src_element: &gst::Element,
        sink_element: &gst::Element,
        pipeline: &gst::Pipeline,
    ) -> Result<(), glib::error::BoolError> {
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
        let sink_element = sink_element.clone();

        // Connect to decodebin's pad-added signal, that is emitted whenever
        // it found another stream from the input file and found a way to decode it to its raw format.
        // decodebin automatically adds a src-pad for this raw stream, which
        // we can use to build the follow-up pipeline.
        src_element.connect_pad_added(move |dbin, src_pad| {
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
                            .static_pad("sink")
                            .expect("queue has no sinkpad");
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
                                .field("error",
                                       ErrorValue(Arc::new(Mutex::new(Some(err)))))
                                .build()
                );
            }
        });
        Ok(())
    }
}

pub fn create_element(source: &SourceType, id: String) -> Result<Element, glib::BoolError> {
    if let Ok(value) = get_uri_type(source) {
        return value.create_element(id);
    }
    if let Ok(value) = get_test_type(source) {
        return value.create_element(id);
    }
    Err(glib::BoolError::new(
        "can't create element",
        "pipeline",
        "func",
        1,
    ))
}

pub fn initialize_element(
    config: &SourceType,
    element: &gst::Element,
    sink: &gst::Element,
    pipeline: &gst::Pipeline,
) -> Result<(), glib::error::BoolError> {
    if let Ok(value) = get_uri_type(config) {
        return value.initialize_element(element, sink, pipeline);
    }
    if let Ok(value) = get_test_type(config) {
        return value.initialize_element(element, sink, pipeline);
    }
    Err(glib::BoolError::new(
        "can't init element",
        "pipeline",
        "func",
        1,
    ))
}

fn get_uri_type(config: &SourceType) -> anyhow::Result<impl SourceTypeConstructor> {
    if let SourceType::URI(uri) = config {
        return Ok(uri);
    }
    Err(anyhow::Error::msg("Could not find constructor Type"))
}

fn get_test_type(config: &SourceType) -> anyhow::Result<impl SourceTypeConstructor> {
    if let SourceType::Test(test) = config {
        return Ok(test);
    }
    Err(anyhow::Error::msg("Could not find constructor Type"))
}
