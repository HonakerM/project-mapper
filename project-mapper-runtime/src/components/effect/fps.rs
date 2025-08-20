use std::sync::mpsc;

use crate::{
    components::{
        branch::BranchControl,
        shared::{Component, ComponentLookupHelper},
    },
    types::message::RuntimeMessage,
};
use anyhow::{Error, Result, anyhow};
use gst::{Element, prelude::*};
use log::{debug, info};
use project_mapper_core::runtime_config::{
    effect::{fps::FpsConfig, gamma::GammaConfig, EffectComponentConfig},
    shared::{ComponentConfig, Uid},
};

pub struct FpsComponent {
    config: EffectComponentConfig,
    element: Element,
    branch: BranchControl,
}

impl FpsComponent {
    fn update_config(element: &gst::Element, config: &FpsConfig) -> Result<()> {
        debug!("Updating fps component with config: {:?}", config);
        if let Some(fps) = &config.max_rate {
            info!("Setting fps element to {}", fps);
            element.set_property("max-rate", fps.clone());
        } else {
            let pspec = element
                .find_property("max-rate")
                .ok_or(anyhow!("Unable to find default fps spec"))?;
            let default_value = pspec.default_value();
            element.set_property("max-rate", &default_value);
        }
        Ok(())
    }
}

impl Component for FpsComponent {
    // runtime lifecycle functions
    // Construct object
    fn new(
        unknown_config: &dyn ComponentConfig,
        pipeline: &gst::Pipeline,
    ) -> Result<FpsComponent> {
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
        let fps_config = match config.config.as_any().downcast_ref::<FpsConfig>() {
            Some(b) => Ok(b.clone()),
            None => Err(anyhow!("GammaComponent is not GammaConfig")),
        }?;
        let element = gst::ElementFactory::make("videorate")
            .name(config.name())
            .build()?;
        FpsComponent::update_config(&element, &fps_config)?;

        let branch = BranchControl::new(config.name(), true, true)?;
        let comp = Self {
            config: config,
            element: element,
            branch: branch,
        };

        // Add elements to the pipelines and sync status
        pipeline.add(&comp.element)?;
        comp.element.sync_state_with_parent()?;

        // ensure the branch is correctly setup and wrap the parent element
        comp.branch.add_to_pipeline(pipeline)?;
        comp.branch.link_wrapped(&comp.element)?;

        Ok(comp)
    }

    // Run any post init setup functions
    // ! Will probably be removed or edited to have more params
    fn setup(
        &mut self,
        pipeline: &gst::Pipeline,
        message_sender: mpsc::Sender<RuntimeMessage>,
        lookup_func: &dyn ComponentLookupHelper,
    ) -> Result<()> {
        // Fetch the compoennt that should be pointing to us and link it to the
        // input queue
        let src_comp =
            lookup_func.lookup_and_setup(self.config.src_uid, pipeline, message_sender.clone())?;

        src_comp
            .borrow()
            .element()?
            .link(self.branch.get_input()?)?;

        Ok(())
    }
    fn update(&mut self, config: &dyn ComponentConfig, _pipeline: &gst::Pipeline) -> Result<()> {
        // parse config and ensure it's correct types
        let config: EffectComponentConfig =
            match config.as_any().downcast_ref::<EffectComponentConfig>() {
                Some(b) => Ok(b.clone()),
                None => Err(Error::msg(
                    "ComponentConfig can not be typed to EffectComponentConfig",
                )),
            }?;

        // construct element
        let fps_config = match config.config.as_any().downcast_ref::<FpsConfig>() {
            Some(b) => Ok(b.clone()),
            None => Err(anyhow!("GammaComponent is not GammaConfig")),
        }?;

        self.config = config;
        FpsComponent::update_config(&self.element, &fps_config)?;

        Ok(())
    }

    // accessor functions
    fn element(&self) -> Result<&Element> {
        // return the branch output element since that's what people
        // should be linking
        self.branch.get_output()
    }
    fn uid(&self) -> Uid {
        return self.config.uid();
    }
}
