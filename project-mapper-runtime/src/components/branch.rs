use crate::{
    components::shared::{Component, ComponentLookupHelper},
    types::message::RuntimeMessage,
};
use anyhow::{Error, Result, anyhow};
use gst::{prelude::*, Element, Event};
use project_mapper_core::runtime_config::shared::{ComponentConfig, Uid};
use std::{sync::mpsc, thread};

/// Controls optional insertion of a queue and/or tee element into a pipeline branch.
pub struct BranchControl {
    name: String,
    pub input_element: Option<Element>,
    pub output_element: Option<Element>,
}

impl BranchControl {
    /// Create a new BranchControl based on flags; `with_queue` and `with_tee`.
    pub fn new(name: String, with_input: bool, with_output: bool) -> Result<Self> {
        let queue = if with_input {
            let q = gst::ElementFactory::make("queue")
                .name(&format!("queue-{}", name))
                .build()?;
            Some(q)
        } else {
            None
        };

        let tee = if with_output {
            let t = gst::ElementFactory::make("tee")
                .name(&format!("tee-{}", name))
                .build()?;
            Some(t)
        } else {
            None
        };

        Ok(Self {
            name: name,
            input_element: queue,
            output_element: tee,
        })
    }

    /// Add configured elements to the pipeline and sync their state.
    pub fn add_to_pipeline(&self, pipeline: &gst::Pipeline) -> Result<()> {
        if let Some(ref q) = self.input_element {
            pipeline.add(q)?;
            q.sync_state_with_parent()?;
        }
        if let Some(ref t) = self.output_element {
            pipeline.add(t)?;
            t.sync_state_with_parent()?;
        }
        Ok(())
    }

    /// Link the optional queue and tee around the provided `element`:
    /// queue -> element -> tee. Returns the tail element for further linking.
    pub fn link_wrapped(&self, element: &Element) -> Result<()> {
        // If queue exists, link queue -> element
        if let Some(ref q) = self.input_element {
            q.link(element)?;
        }
        // If tee exists, link element -> tee and return tee
        if let Some(ref t) = self.output_element {
            element.link(t)?;
        }
        Ok(())
    }

    pub fn destory(&self) -> Result<()> {
        if let Some(input_element) = &self.input_element {
            BranchControl::unlink_and_destory(input_element)?;
        }
        if let Some(output_element) = &self.output_element {
            BranchControl::unlink_and_destory(output_element)?;
        }
        Ok(())
    }
    pub fn get_output(&self) -> Result<&Element> {
        self.output_element
            .as_ref()
            .ok_or_else(|| anyhow!("Input element does not exist for branch: {}", self.name))
    }

    pub fn link_from(&self, element: &Element) -> Result<()> {
        // Link the provided element to the input element of the branch
        let input_element = self
            .input_element
            .as_ref()
            .ok_or_else(|| anyhow!("Input element does not exist for branch: {}", self.name))?;
        element.link(input_element)?;

        Ok(())
    }

    pub fn get_input(&self) -> Result<&Element> {
        self.input_element
            .as_ref()
            .ok_or_else(|| anyhow!("Input element does not exist for branch: {}", self.name))
    }

    pub fn unlink_element(element: &Element, pipeline: &gst::Pipeline) -> Result<()> {
        for input_sink_pad in element.sink_pads() {
            if let Some(input_src_pad) = input_sink_pad.peer() {

                input_src_pad.probes
                let local_element = element.clone();
                let local_pipeline = pipeline.clone();
                input_src_pad.add_probe(gst::PadProbeType::BLOCK, move |pad, info| {
                    if let Some(input_element) = pad.parent_element() {
                        input_element.set_state(gst::State::Paused).unwrap();
                    }
                    pad.unlink(&input_sink_pad).unwrap();
                    
                    let (sender, receiver) = mpsc::channel();
                    let mut total_count = 0;
                    // look for a src and then wait for eos event
                    for src_element in local_element.src_pads() {
                        total_count+=1;
                        // Add a probe to the pad
                        let eos_local_element = local_element.clone();
                        let local_sender = sender.clone();
                        let current_count = total_count.clone();
                        src_element.add_probe(gst::PadProbeType::EVENT_DOWNSTREAM, move |src_pad, probe_info| {
                            if let Some(event) = probe_info.event() {
                                // Filter for the specific event type
                                if event.type_() == gst::EventType::Eos {
                                    println!("Received EOS event on the main sink pad!");

                                    if let Some(peer_sink) = &src_pad.peer() {
                                        src_pad.unlink(peer_sink);
                                    }
                                    local_sender.send(current_count);
                                    return gst::PadProbeReturn::Remove; // Remove the probe after the event
                                }
                            }
                            gst::PadProbeReturn::Pass // Pass the event downstream
                        });
                    }

                    let eos_event = gst::event::Eos::new();
                    local_element.send_event(eos_event);
                    // once all eos's have been received stop and remove the element
                    for _ in receiver.iter() {
                        total_count-=1;
                        if total_count <= 0 {
                            break;
                        }
                    }
                    local_element.set_state(gst::State::Null);
                    local_pipeline.remove(&local_element);

                    gst::PadProbeReturn::Ok
                });
            }
        }
        Ok(())
    }
    pub fn unlink_and_destory(element: &Element, pipeline: &gst::Pipeline) -> Result<()> {
        BranchControl::unlink_element(element, pipeline)?;
        element.set_state(gst::State::Null)?;
        Ok(())
    }

    pub fn link_to(&self, element: &Element) -> Result<()> {
        let output_element = self
            .output_element
            .as_ref()
            .ok_or_else(|| anyhow!("Output element does not exist for branch: {}", self.name))?;
        // Link the output element of the branch to the provided element
        output_element.link(element)?;

        Ok(())
    }
}
