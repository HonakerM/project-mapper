use std::sync::mpsc;

use crate::{
    components::{
        bin_wrapper::BinWrapper, branch::BranchControl, shared::{Component, ComponentLookupHelper}
    },
    types::message::RuntimeMessage, utils::gstreamer::unlink_element,
};
use anyhow::{Error, Result, anyhow};
use gst::{prelude::*, Element, Pipeline, State};
use log::{debug, info};
use project_mapper_core::runtime_config::{
    effect::{EffectComponentConfig, fps::FpsConfig, gamma::GammaConfig},
    shared::{ComponentConfig, Uid},
};

pub struct FpsComponent {
    config: EffectComponentConfig,
    rate_element: Element,
    bin_element: BinWrapper,
    pipeline: Option<Pipeline>,
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
    fn new(unknown_config: &dyn ComponentConfig) -> Result<FpsComponent> {
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
        let rate_element = gst::ElementFactory::make("videorate")
            .name(config.name())
            .build()?;
        FpsComponent::update_config(&rate_element, &fps_config)?;

        let bin_wrapper = BinWrapper::new(&[&rate_element], true, true);

        let comp = Self {
            config: config,
            rate_element: rate_element,
            bin_element: bin_wrapper,
            pipeline: None,
        };

        Ok(comp)
    }

    // Run any post init setup functions
    // ! Will probably be removed or edited to have more params
    fn setup(
        &mut self,
        pipeline: &gst::Pipeline,
        _message_sender: mpsc::Sender<RuntimeMessage>,
    ) -> Result<()> {
        // Add elements to the pipelines and sync status
        self.pipeline = Some(pipeline.clone());
        pipeline.add(&self.bin_element)?;
        self.bin_element.sync_state_with_parent()?;

        Ok(())
    }
    fn update_and_link(
        &mut self,
        config: &dyn ComponentConfig,
        lookup_func: &dyn ComponentLookupHelper,
    ) -> Result<()> {
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
        FpsComponent::update_config(&self.rate_element, &fps_config)?;

        if self.config.srcs.len() != 1 {
            return Err(anyhow!("Balance component must have exactly one source"));
        }
        let src_config = &self.config.srcs[0];

        let src_comp = lookup_func.get_comp(&src_config.uid()).ok_or(anyhow!(
            "Unable to find source component {} for fps component {}",
            src_config.uid(),
            self.config.name()
        ))?;
        let src_comp_ref = src_comp.borrow();
        let src_element = src_comp_ref.output_element()?;

        //if  self.input_element()?.current_state() == State::Playing  && let Some(pipeline) = &self.pipeline {
        if  let Some(pipeline) = &self.pipeline {
            unlink_element(self.input_element()?, pipeline)?;
        }
        src_element.link(self.input_element()?)?;
        

        Ok(())
    }

    // accessor functions
    fn input_element(&self) -> Result<&Element> {
        // return the branch output element since that's what people
        // should be linking
        Ok(&self.bin_element.upcast_ref::<gst::Element>())
    }
    fn output_element(&self) -> Result<&Element> {
        Ok(&self.bin_element.upcast_ref::<gst::Element>())
    }
    fn uid(&self) -> Uid {
        return self.config.uid();
    }
}
