use crate::{
    components::shared::{Component, ComponentLookupHelper},
    types::message::RuntimeMessage,
};
use anyhow::{Error, Result, anyhow};
use gst::{Element, prelude::*};
use project_mapper_core::runtime_config::{
    effect::{EffectComponentConfig, common::EffectConfig},
    shared::{ComponentConfig, Uid},
};
use std::sync::mpsc;

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

    pub fn get_output(&self) -> Result<&Element> {
        self.output_element
            .as_ref()
            .ok_or_else(|| anyhow!("Input element does not exist for branch: {}", self.name))
    }

    pub fn get_input(&self) -> Result<&Element> {
        self.input_element
            .as_ref()
            .ok_or_else(|| anyhow!("Input element does not exist for branch: {}", self.name))
    }
}
