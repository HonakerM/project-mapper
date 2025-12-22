use std::{
    any::type_name_of_val,
    collections::{HashMap, HashSet},
    error::Error,
    fmt::Display,
    ops::RangeTo,
};

use serde::{Deserialize, Serialize};

use crate::{
    available_config::{config::AvailableConfigTrait, utils::insert_type_into_config},
    runtime_config::{
        effect::EffectComponentConfig,
        input::InputComponentConfig,
        output::{OutputComponentConfig, common::OutputConfigTrait},
        shared::{ComponentConfig, Uid},
        utils::{changes::RuntimeConfigChangeTracker, validation::gather_validation_helper_data},
    },
    types::{errors::RuntimeConfigValidationError, openapi::OpenAPISchema},
};
use anyhow::Result as AnyhowResult;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AvailableOutputConfig {
    pub type_name: String,
    pub config_schema: OpenAPISchema,
    pub requires_refresh: bool,
}

impl AvailableOutputConfig {
    pub const fn new(
        type_name: String,
        config_schema: OpenAPISchema,
        requires_refresh: bool,
    ) -> Self {
        Self {
            type_name,
            config_schema,
            requires_refresh,
        }
    }

    pub fn from_output_config(config: Box<dyn OutputConfigTrait>, schema: OpenAPISchema) -> Self {
        Self {
            type_name: config.typetag_name().to_string(),
            config_schema: schema,
            requires_refresh: false,
        }
    }

    pub fn schema(&self) -> OpenAPISchema {
        let mut local_schema = self.config_schema.to_json_value();
        insert_type_into_config(&mut local_schema, self.type_name.clone());
        OpenAPISchema::try_from(local_schema).unwrap()
    }
}

impl AvailableConfigTrait for AvailableOutputConfig {
    fn requires_refresh(&self) -> bool {
        self.requires_refresh
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
