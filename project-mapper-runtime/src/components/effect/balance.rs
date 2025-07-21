use std::sync::mpsc;

use crate::{
    components::shared::{Component, ComponentLookupHelper},
    types::message::RuntimeMessage,
};
use anyhow::{Error, Result, anyhow};
use gst::{Element, prelude::*};
use project_mapper_core::runtime_config::{
    effect::{EffectComponentConfig, common::EffectConfig},
    input::{InputComponentConfig, common::InputConfig},
    shared::{ComponentConfig, Uid},
};

pub struct BalanceComponent {
    config: EffectComponentConfig,
    element: Element,
    queue_element: Element,
    tee_element: Element,
    has_setup: bool,
}

impl Component for BalanceComponent {
    // runtime lifecycle functions
    // Construct object
    fn new(unknown_config: &dyn ComponentConfig) -> Result<BalanceComponent> {
        // parse config and ensure it's correct types
        let config: EffectComponentConfig = match unknown_config
            .as_any()
            .downcast_ref::<EffectComponentConfig>()
        {
            Some(b) => Ok(b.clone()),
            None => Err(Error::msg(
                "ComponentConfig can not be typed to EffectComponentConfig",
            )),
        }?;

        // construct element
        let element = match &config.config {
            EffectConfig::Balance(balance_config) => {
                let element = gst::ElementFactory::make("videobalance")
                    .name(config.name())
                    .build()?;

                if let Some(brightness) = &balance_config.brightness {
                    element.set_property("brightness", brightness.clone());
                }
                if let Some(contrast) = &balance_config.contrast {
                    element.set_property("contrast", contrast.clone());
                }
                if let Some(saturation) = &balance_config.saturation {
                    element.set_property("saturation", saturation.clone());
                }
                if let Some(hue) = &balance_config.hue {
                    element.set_property("hue", hue.clone());
                }
                Ok(element)
            }
            _ => Err(anyhow!("Component Config is not proper type")),
        }?;

        // Add tee to src element to allow multiple linkages
        let tee_name: String = format!("tee-{}", config.name());
        let sink_tee = gst::ElementFactory::make("tee").name(tee_name).build()?;

        let src_queue = gst::ElementFactory::make("queue")
            .name(format!("queue-{}", config.name()))
            .build()?;

        Ok(Self {
            config: config,
            element: element,
            tee_element: sink_tee,
            queue_element: src_queue,
            has_setup: false,
        })
    }

    // Run any post init setup functions
    // ! Will probably be removed or edited to have more params
    fn setup(
        &mut self,
        pipeline: &gst::Pipeline,
        message_sender: mpsc::Sender<RuntimeMessage>,
        lookup_func: &dyn ComponentLookupHelper,
    ) -> Result<()> {
        self.has_setup = true;

        // Add elements to the pipelines and sync status
        pipeline.add(&self.element)?;
        pipeline.add(&self.tee_element)?;
        pipeline.add(&self.queue_element)?;

        self.element.sync_state_with_parent()?;
        self.tee_element.sync_state_with_parent()?;
        self.queue_element.sync_state_with_parent()?;

        self.queue_element.link(&self.element)?;
        self.element.link(&self.tee_element)?;

        // Fetch the compoennt that should be pointing to us and link it to the
        // input queue
        let src_comp =
            lookup_func.lookup_and_setup(self.config.src_uid, pipeline, message_sender.clone())?;
        src_comp.borrow().element()?.link(&self.queue_element)?;

        Ok(())
    }

    // accessor functions
    fn element(&self) -> Result<&Element> {
        // return the tee element since that's what people should
        // be linking against
        Ok(&self.tee_element)
    }
    fn uid(&self) -> Uid {
        return self.config.uid();
    }
    fn has_setup(&self) -> bool {
        return self.has_setup;
    }
}
