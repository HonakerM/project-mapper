use std::sync::mpsc;

use anyhow::{Error, Result, anyhow};
use log::{debug, info};
use project_mapper_core::runtime_config::{
    effect::EffectComponentConfig,
    shared::{ComponentConfig, Uid},
};
use project_mapper_runtime::gst::{Element, prelude::*};
use project_mapper_runtime::{
    components::{
        branch::BranchControl,
        shared::{Component, ComponentLookupHelper},
    },
    gst,
    types::message::RuntimeMessage,
};

pub struct FpsComponent {
    config: EffectComponentConfig,
    element: Element,
    branch: BranchControl,
}

use std::any::Any;

use serde::{Deserialize, Deserializer, Serialize, de};

use project_mapper_core::runtime_config::{
    effect::common::EffectConfigTrait, utils::validation::ensure_config_bounds,
};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct FpsConfig {
    pub max_rate: Option<i32>,
}

// Implement InputConfigTrait for TestConfig
#[typetag::serde]
impl EffectConfigTrait for FpsConfig {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn EffectConfigTrait> {
        Box::new(self.clone())
    }
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

        // construct element
        let fps_config = match config.config.as_any().downcast_ref::<FpsConfig>() {
            Some(b) => Ok(b.clone()),
            None => Err(anyhow!("GammaComponent is not GammaConfig")),
        }?;

        self.config = config;
        FpsComponent::update_config(&self.element, &fps_config)?;

        if self.config.srcs.len() != 1 {
            return Err(anyhow!("Balance component must have exactly one source"));
        }

        Ok(())
    }

    // accessor functions
    fn input_element(&self) -> Option<&Element> {
        // return the branch output element since that's what people
        // should be linking
        self.branch.get_input()
    }
    fn output_element(&self) -> Option<&Element> {
        self.branch.get_output()
    }
    fn uid(&self) -> Uid {
        return self.config.uid();
    }
}
