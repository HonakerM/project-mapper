use std::sync::mpsc;

use anyhow::{Error, Result, anyhow};
use log::debug;
use project_mapper_core::{
    runtime_config::{
        effect::EffectComponentConfig,
        shared::{ComponentConfig, Uid},
    },
    types::openapi::OpenAPISchema,
};
use schemars::{JsonSchema, schema_for};

use project_mapper_runtime::gst::{Element, prelude::*};
use project_mapper_runtime::{
    components::{
        branch::BranchControl,
        shared::{Component, ComponentLookupHelper},
    },
    gst,
    types::message::RuntimeMessage,
};

use std::any::Any;

use serde::{Deserialize, Deserializer, Serialize, de};

use project_mapper_core::runtime_config::{
    effect::common::EffectConfigTrait, utils::validation::ensure_config_bounds,
};

#[derive(Serialize, Deserialize, Debug, Default, Clone, JsonSchema)]
#[serde(default)]
pub struct BalanceConfig {
    #[serde(deserialize_with = "deserialize_bounded_brightness")]
    pub brightness: Option<f64>,
    #[serde(deserialize_with = "deserialize_bounded_contrast")]
    pub contrast: Option<f64>,
    #[serde(deserialize_with = "deserialize_bounded_hue")]
    pub hue: Option<f64>,
    #[serde(deserialize_with = "deserialize_bounded_saturation")]
    pub saturation: Option<f64>,
}

// Implement InputConfigTrait for TestConfig
#[typetag::serde]
impl EffectConfigTrait for BalanceConfig {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn EffectConfigTrait> {
        Box::new(self.clone())
    }
}

impl BalanceConfig {
    pub fn openapi_schema() -> OpenAPISchema {
        let mut schema_val = serde_json::to_value(schema_for!(BalanceConfig)).unwrap();
        match schema_val.as_object_mut() {
            Some(map) => {
                map.insert(
                    "description".to_string(),
                    serde_json::Value::String("Requires src type `default`".to_string()),
                );
            }
            None => {}
        };
        schema_val.try_into().unwrap()
    }
}

fn deserialize_bounded_brightness<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let some_val = Option::<f64>::deserialize(deserializer)?;
    ensure_config_bounds(some_val, -1.0, 1.0).map_err(|e| {
        de::Error::custom(format!(
            "brightness value {:?} has error: {:#}",
            some_val, e,
        ))
    })
}

fn deserialize_bounded_contrast<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let some_val = Option::<f64>::deserialize(deserializer)?;
    ensure_config_bounds(some_val, 0.0, 2.0).map_err(|e| {
        de::Error::custom(format!("contrast value {:?} has error: {:#}", some_val, e,))
    })
}

fn deserialize_bounded_hue<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let some_val = Option::<f64>::deserialize(deserializer)?;
    ensure_config_bounds(some_val, -1.0, 1.0)
        .map_err(|e| de::Error::custom(format!("hue value {:?} has error: {:#}", some_val, e,)))
}

fn deserialize_bounded_saturation<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let some_val = Option::<f64>::deserialize(deserializer)?;
    ensure_config_bounds(some_val, 0.0, 2.0).map_err(|e| {
        de::Error::custom(format!(
            "saturation value {:?} has error: {:#}",
            some_val, e,
        ))
    })
}

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

#[project_mapper_macro::effect_component(config = {BalanceConfig::default()}, schema = {BalanceConfig::openapi_schema().to_json_value()})]
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
