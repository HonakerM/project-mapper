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
use log::debug;
use project_mapper_core::runtime_config::{
    effect::{EffectComponentConfig, balance::BalanceConfig},
    shared::{ComponentConfig, Uid},
};

pub struct BalanceComponent {
    config: EffectComponentConfig,
    element: Element,
    branch: BranchControl,
}

impl BalanceComponent {
    fn update_config(element: &gst::Element, config: BalanceConfig) -> Result<()> {
        debug!("Updating balance component with config: {:?}", config);
        if let Some(brightness) = &config.brightness {
            element.set_property("brightness", brightness.clone());
        } else {
            let pspec = element
                .find_property("brightness")
                .ok_or(anyhow!("Unable to find default brightness spec"))?;
            let default_value = pspec.default_value();
            element.set_property("brightness", &default_value);
        }
        if let Some(contrast) = &config.contrast {
            element.set_property("contrast", contrast.clone());
        } else {
            let pspec = element
                .find_property("brightness")
                .ok_or(anyhow!("Unable to find default brightness spec"))?;
            let default_value = pspec.default_value();
            element.set_property("brightness", &default_value);
        }
        if let Some(saturation) = &config.saturation {
            element.set_property("saturation", saturation.clone());
        } else {
            let pspec = element
                .find_property("saturation")
                .ok_or(anyhow!("Unable to find default saturation spec"))?;
            let default_value = pspec.default_value();
            element.set_property("saturation", &default_value);
        }
        if let Some(hue) = &config.hue {
            element.set_property("hue", hue.clone());
        } else {
            let pspec = element
                .find_property("hue")
                .ok_or(anyhow!("Unable to find default hue spec"))?;
            let default_value = pspec.default_value();
            element.set_property("hue", &default_value);
        }
        Ok(())
    }
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
        let balance_config = match config.config.as_any().downcast_ref::<BalanceConfig>() {
            Some(b) => Ok(b.clone()),
            None => Err(anyhow!("BalannceComponentConfig is not BalanceConfig")),
        }?;
        let element = gst::ElementFactory::make("videobalance")
            .name(config.name())
            .build()?;
        BalanceComponent::update_config(&element, balance_config)?;

        let branch = BranchControl::new(config.name(), true, true)?;
        let comp = Self {
            config: config,
            element: element,
            branch: branch,
        };

        Ok(comp)
    }

    // Run any post init setup functions
    // ! Will probably be removed or edited to have more params
    fn setup(
        &mut self,
        pipeline: &gst::Pipeline,
        message_sender: mpsc::Sender<RuntimeMessage>,
    ) -> Result<()> {
        // config the elements in the pipeline
        pipeline.add(&self.element)?;
        self.element.sync_state_with_parent()?;

        // ensure the branch is correctly setup and wrap the parent element
        self.branch.add_to_pipeline(pipeline)?;
        self.branch.link_wrapped(&self.element)?;

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

        // update config
        let balance_config = match config.config.as_any().downcast_ref::<BalanceConfig>() {
            Some(b) => Ok(b.clone()),
            None => Err(anyhow!("BalannceComponentConfig is not BalanceConfig")),
        }?;
        BalanceComponent::update_config(&self.element, balance_config)?;

        if config.srcs.len() != 1 {
            return Err(anyhow!("Balance component must have exactly one source"));
        }

        self.config = config;
        Ok(())
    }

    // accessor functions
    fn input_element(&self) -> Result<&Element> {
        // return the branch output element since that's what people
        // should be linking
        self.branch.get_input()
    }
    fn output_element(&self) -> Result<&Element> {
        self.branch.get_output()
    }
    fn uid(&self) -> Uid {
        return self.config.uid();
    }
}
