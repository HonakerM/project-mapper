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
    effect::{EffectComponentConfig, gamma::GammaConfig},
    shared::{ComponentConfig, Uid},
};

pub struct GammaComponent {
    config: EffectComponentConfig,
    element: Element,
    branch: BranchControl,
}

impl GammaComponent {
    fn update_config(element: &gst::Element, config: &GammaConfig) -> Result<()> {
        debug!("Updating gamma component with config: {:?}", config);
        if let Some(gamma) = &config.gamma {
            info!("Setting gamma element to {}", gamma);
            element.set_property("gamma", gamma.clone());
        } else {
            let pspec = element
                .find_property("gamma")
                .ok_or(anyhow!("Unable to find default gamma spec"))?;
            let default_value = pspec.default_value();
            element.set_property("gamma", &default_value);
        }
        Ok(())
    }
}
impl Component for GammaComponent {
    // runtime lifecycle functions
    // Construct object
    fn new(unknown_config: &dyn ComponentConfig) -> Result<GammaComponent> {
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
        let gamma_config = match config.config.as_any().downcast_ref::<GammaConfig>() {
            Some(b) => Ok(b.clone()),
            None => Err(anyhow!("GammaComponent is not GammaConfig")),
        }?;
        let element = gst::ElementFactory::make("gamma")
            .name(config.name())
            .build()?;
        GammaComponent::update_config(&element, &gamma_config)?;

        let branch = BranchControl::new(config.name(), true, true)?;
        Ok(Self {
            config: config,
            element: element,
            branch: branch,
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
        // Add elements to the pipelines and sync status
        pipeline.add(&self.element)?;
        self.element.sync_state_with_parent()?;

        // ensure the branch is correctly setup and wrap the parent element
        self.branch.add_to_pipeline(pipeline)?;
        self.branch.link_wrapped(&self.element)?;

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
    fn update(&mut self, config: &dyn ComponentConfig) -> Result<()> {
        // parse config and ensure it's correct types
        let config: EffectComponentConfig =
            match config.as_any().downcast_ref::<EffectComponentConfig>() {
                Some(b) => Ok(b.clone()),
                None => Err(Error::msg(
                    "ComponentConfig can not be typed to EffectComponentConfig",
                )),
            }?;

        // construct element
        let gamma_config = match config.config.as_any().downcast_ref::<GammaConfig>() {
            Some(b) => Ok(b.clone()),
            None => Err(anyhow!("GammaComponent is not GammaConfig")),
        }?;

        self.config = config;
        GammaComponent::update_config(&self.element, &gamma_config)?;

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
