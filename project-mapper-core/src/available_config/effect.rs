use std::{
    any::type_name_of_val,
    collections::{HashMap, HashSet},
    error::Error,
    fmt::Display,
    ops::RangeTo,
};

use serde::{Deserialize, Serialize};

use crate::{
    available_config::config::AvailableConfigTrait,
    runtime_config::{
        effect::{
            EffectComponentConfig,
            common::{EffectConfigTrait, EffectSrcConfigTrait},
        },
        input::InputComponentConfig,
        output::OutputComponentConfig,
        shared::{ComponentConfig, Uid},
        utils::{changes::RuntimeConfigChangeTracker, validation::gather_validation_helper_data},
    },
    types::{errors::RuntimeConfigValidationError, openapi::OpenAPISchema},
};
use anyhow::Result as AnyhowResult;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AvailableEffectConfig {
    pub config_name: String,
    pub config_schema: OpenAPISchema,
    pub src_name: String,
    pub src_schema: OpenAPISchema,
    pub requires_refresh: bool,
}

impl AvailableEffectConfig {
    pub const fn new(
        config_name: String,
        config_schema: OpenAPISchema,
        src_name: String,
        src_schema: OpenAPISchema,
        requires_refresh: bool,
    ) -> Self {
        Self {
            config_name,
            src_name,
            config_schema,
            src_schema,
            requires_refresh,
        }
    }

    pub fn from_effect_config(
        config: &dyn EffectConfigTrait,
        src: &dyn EffectSrcConfigTrait,
        config_schema: OpenAPISchema,
        src_schema: OpenAPISchema,
    ) -> Self {
        Self {
            src_name: src.typetag_name().to_string(),
            src_schema: src_schema,
            config_name: config.typetag_name().to_string(),
            config_schema: config_schema,
            requires_refresh: false,
        }
    }
}

impl AvailableConfigTrait for AvailableEffectConfig {
    fn requires_refresh(&self) -> bool {
        self.requires_refresh
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
