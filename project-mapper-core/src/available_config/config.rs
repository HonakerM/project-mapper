use std::{
    any::type_name_of_val,
    collections::{HashMap, HashSet},
    error::Error,
    fmt::Display,
    ops::RangeTo,
};

use serde::{Deserialize, Serialize};

use crate::{
    available_config::{
        effect::AvailableEffectConfig,
        input::AvailableInputConfig,
        output::AvailableOutputConfig,
        utils::{construct_base_schema, insert_config_into_base},
    },
    runtime_config::{
        effect::EffectComponentConfig,
        input::InputComponentConfig,
        output::OutputComponentConfig,
        shared::{ComponentConfig, Uid},
        utils::{changes::RuntimeConfigChangeTracker, validation::gather_validation_helper_data},
    },
    types::{errors::RuntimeConfigValidationError, openapi::OpenAPISchema},
};
use anyhow::Result as AnyhowResult;

pub trait AvailableConfigTrait {
    fn as_any(&self) -> &dyn std::any::Any;
    fn requires_refresh(&self) -> bool;
}

// Top-Level Config object for the runtime
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AvailableConfig {
    pub inputs: Vec<AvailableInputConfig>,
    pub effects: Vec<AvailableEffectConfig>,
    pub outputs: Vec<AvailableOutputConfig>,
}

impl AvailableConfig {
    pub fn get_schema(&self) -> OpenAPISchema {
        let input_schema = {
            let mut base_schema = construct_base_schema().to_json_value();

            let mut dynamic_schemas = vec![];
            for input_config in &self.inputs {
                dynamic_schemas.push(input_config.schema().to_json_value());
            }
            println!("Schemas, {:?}", dynamic_schemas);

            insert_config_into_base(
                &mut base_schema,
                "config".to_owned(),
                serde_json::json!({"oneOf":dynamic_schemas}),
            );
            base_schema
        };
        let effect_schema = {
            let mut base_schema = construct_base_schema().to_json_value();

            let mut config_schemas = vec![];
            let mut src_schemas = vec![];
            for input_config in &self.effects {
                config_schemas.push(input_config.config_schema().to_json_value());
                src_schemas.push(input_config.src_schema().to_json_value());
            }

            insert_config_into_base(
                &mut base_schema,
                "config".to_owned(),
                serde_json::json!({"oneOf":config_schemas}),
            );
            insert_config_into_base(
                &mut base_schema,
                "src".to_owned(),
                serde_json::json!({"oneOf":src_schemas}),
            );
            base_schema
        };
        let output_schema = {
            let mut base_schema = construct_base_schema().to_json_value();

            let mut dynamic_schemas = vec![];
            for input_config in &self.outputs {
                dynamic_schemas.push(input_config.schema().to_json_value());
            }

            insert_config_into_base(
                &mut base_schema,
                "config".to_owned(),
                serde_json::json!({"oneOf":dynamic_schemas}),
            );
            base_schema
        };

        serde_json::json!({
            "type":"object",
            "parameters": {
                "inputs": input_schema,
                "effects":effect_schema,
                "outputs":output_schema,
            }
        })
        .try_into()
        .unwrap()
    }
}

pub enum AvailableConfigType {
    Input(AvailableInputConfig),
    Effect(AvailableEffectConfig),
    Output(AvailableOutputConfig),
}
