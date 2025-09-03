use gst::{prelude::*, PadProbeType};
use gst::subclass::prelude::*;
use gst::{glib, Element, Bin, GhostPad, Pad};
use std::sync::{mpsc, Mutex};
use anyhow::Result;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct BinWrapper {
        pub queue: Mutex<Option<Element>>,
        pub tee: Mutex<Option<Element>>,
        pub elements: Mutex<Vec<Element>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BinWrapper {
        const NAME: &'static str = "BinWrapper";
        type Type = super::BinWrapper;
        type ParentType = gst::Bin;
    }

    impl ObjectImpl for BinWrapper {}
    impl GstObjectImpl for BinWrapper {}
    impl ElementImpl for BinWrapper {
        fn metadata() -> Option<&'static gst::subclass::ElementMetadata> {
            static ELEMENT_METADATA: std::sync::OnceLock<gst::subclass::ElementMetadata> =
            std::sync::OnceLock::new();

            Some(ELEMENT_METADATA.get_or_init(|| {
                gst::subclass::ElementMetadata::new(
                    "BinWrapper",
                    "Bin/Custom",
                    "Wraps multiple elements with queue+tee",
                    "Your Name <you@example.com>",
                )
            }))
        }

        fn pad_templates() -> &'static [gst::PadTemplate] {
            use once_cell::sync::Lazy;
            static PAD_TEMPLATES: Lazy<Vec<gst::PadTemplate>> = Lazy::new(|| {
                vec![
                    gst::PadTemplate::new(
                        "sink",
                        gst::PadDirection::Sink,
                        gst::PadPresence::Always,
                        &gst::Caps::new_any(),
                    )
                    .unwrap(),
                    gst::PadTemplate::new(
                        "src_%u",
                        gst::PadDirection::Src,
                        gst::PadPresence::Request,
                        &gst::Caps::new_any(),
                    )
                    .unwrap(),
                ]
            });
            PAD_TEMPLATES.as_ref()
        }

        fn request_new_pad(
                &self,
                templ: &gst::PadTemplate,
                name: Option<&str>,
                caps: Option<&gst::Caps>,
        ) -> Option<gst::Pad> {
            // Try to fetch internal tee
            let tee_lock = self.tee.lock().unwrap();
            let tee = tee_lock.as_ref();
            if let Some(tee_elm) = tee {
                let tee_pad = tee_elm.request_pad_simple("src_%u")?;
                let ghost = GhostPad::with_target(&tee_pad).ok()?;
                ghost.set_active(true).ok()?;
    
                let element = self.obj();
                element.add_pad(&ghost).ok()?;
                Some(ghost.upcast())
            } else {
                let element_vec_lock = self.elements.lock().unwrap();
                let first_element = &element_vec_lock[0];
                let element_pad = first_element.request_pad_simple("src_%u")?;
                let ghost = GhostPad::with_target(&element_pad).ok()?;
                ghost.set_active(true).ok()?;
    
                let element = self.obj();
                element.add_pad(&ghost).ok()?;
                Some(ghost.upcast())

            }

        }

        fn release_pad(&self, pad: &gst::Pad) {
            if let Some(ghost_pad) = pad.downcast_ref::<gst::GhostPad>() {
                if let Some(target) = ghost_pad.target() {
                    if let Some(tee) = self.tee.lock().unwrap().as_ref() {
                        // release the request pad on the internal tee
                        tee.release_request_pad(&target);
                    }
                }
            }
             let element = self.obj();
             element.remove_pad(pad);
         }
    }
    impl BinImpl for BinWrapper {}
}

glib::wrapper! {
    pub struct BinWrapper(ObjectSubclass<imp::BinWrapper>)
        @extends gst::Bin, gst::Element, gst::Object;
}

impl BinWrapper {
    /// Create a new BinWrapper with a list of elements to insert between queue → ... → tee
    pub fn new(elements: &[&Element], with_input: bool, with_output: bool) -> Self {
        let this: Self = glib::Object::builder().build();

        {
            let mut chain: Vec<Element> = vec![];
            let mut queue_lock = this.imp().queue.lock().unwrap();
            let mut tee_lock = this.imp().tee.lock().unwrap();

            // Input wrapper
            if with_input {
                let queue = gst::ElementFactory::make("queue").build().unwrap();
                *queue_lock = Some(queue);
                if let Some(element) = queue_lock.as_ref() {
                    chain.push(element.clone());
                }
            }


            // Collect full chain: queue → elements... → tee
            for e in elements {
                chain.push((*e).clone());
            }

            if with_output {
                let tee = gst::ElementFactory::make("tee").build().unwrap();
                *tee_lock = Some(tee.clone());
                if let Some(element) = tee_lock.as_ref() {
                    chain.push(element.clone());
                }
            }

            this.add_many(&chain).unwrap();
            gst::Element::link_many(&chain).unwrap();
            this.imp().elements.lock().unwrap().extend(chain);

            // Get the queue if we have input
            if let Some(element) = queue_lock.as_ref() {
                let sink_pad = element.static_pad("sink").unwrap();
                let ghost_sink = GhostPad::with_target(&sink_pad).unwrap();
                ghost_sink.set_active(true).unwrap();
                this.add_pad(&ghost_sink).unwrap();
            }
        }

        this
        
    }
}
