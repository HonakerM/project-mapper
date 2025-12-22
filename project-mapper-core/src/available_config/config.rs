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
        effect::AvailableEffectConfig, input::AvailableInputConfig, output::AvailableOutputConfig,
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

pub enum AvailableConfigType {
    Input(AvailableInputConfig),
    Effect(AvailableEffectConfig),
    Output(AvailableOutputConfig),
}
