use crate::{
    components::shared::{Component, ComponentLookupHelper},
    types::message::RuntimeMessage,
};
use anyhow::{Error, Result, anyhow};
use gst::{Element, prelude::*};
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

    pub fn unlink_from_input(&self) -> Result<()> {
        let input_element = self
            .input_element
            .as_ref()
            .ok_or_else(|| anyhow!("Input element does not exist for branch: {}", self.name))?;
        if let Some(input_sink_pad) = input_element.static_pad("sink") {
            if input_sink_pad.is_linked() {
                if let Some(peer_pad) = input_sink_pad.peer() {
                    peer_pad.unlink(&input_sink_pad)?;
                }
            }
        }
        Ok(())
    }
    pub fn link_from(&self, element: &Element) -> Result<()> {
        self.unlink_from_input()?;

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

    pub fn unlink_element(element: &Element) -> Result<()> {
        for sink_pad in element.sink_pads() {
            if sink_pad.is_linked() {
                if let Some(peer_pad) = sink_pad.peer() {
                    peer_pad.unlink(&sink_pad)?;
                }
            }
        }
        for src_pad in element.src_pads() {
            if src_pad.is_linked() {
                if let Some(peer_pad) = src_pad.peer() {
                    src_pad.unlink(&peer_pad)?;
                }
            }
        }
        Ok(())
    }
    pub fn unlink_and_destory(element: &Element) -> Result<()> {
        BranchControl::unlink_element(element)?;
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
