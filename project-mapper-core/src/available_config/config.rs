use std::{
    any::type_name_of_val,
    collections::{HashMap, HashSet},
    error::Error,
    fmt::Display,
    ops::RangeTo,
};

use schemars::Schema;
use serde::{Deserialize, Serialize};

use crate::{
    runtime_config::{
        effect::EffectComponentConfig,
        input::InputComponentConfig,
        output::OutputComponentConfig,
        shared::{ComponentConfig, Uid},
        utils::{changes::RuntimeConfigChangeTracker, validation::gather_validation_helper_data},
    },
    types::errors::RuntimeConfigValidationError,
};
use anyhow::Result as AnyhowResult;

pub static DEFAULT_ID: Uid = -1;
pub static DEFAULT_NAME: &str = "DefaultRuntimeComponent";
pub static UNUSED_RANGE: RangeTo<Uid> = (..0);

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

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AvailableInputConfig {
    pub type_name: String,
    pub config_schema: Schema,
    pub requires_refresh: bool,
}

impl AvailableInputConfig {
    pub const fn new(type_name: String, config_schema: Schema, requires_refresh: bool) -> Self {
        Self {
            type_name,
            config_schema,
            requires_refresh,
        }
    }
}

impl AvailableConfigTrait for AvailableInputConfig {
    fn requires_refresh(&self) -> bool {
        self.requires_refresh
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AvailableEffectConfig {
    pub type_name: String,
    pub config_schema: Schema,
    pub src_schema: Schema,
    pub requires_refresh: bool,
}

impl AvailableEffectConfig {
    pub const fn new(
        type_name: String,
        config_schema: Schema,
        src_schema: Schema,
        requires_refresh: bool,
    ) -> Self {
        Self {
            type_name,
            config_schema,
            src_schema,
            requires_refresh,
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

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AvailableOutputConfig {
    pub type_name: String,
    pub config_schema: Schema,
    pub requires_refresh: bool,
}

impl AvailableOutputConfig {
    pub const fn new(type_name: String, config_schema: Schema, requires_refresh: bool) -> Self {
        Self {
            type_name,
            config_schema,
            requires_refresh,
        }
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

pub enum AvailableConfigType {
    Input(AvailableInputConfig),
    Effect(AvailableEffectConfig),
    Output(AvailableOutputConfig),
}
