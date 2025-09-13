use std::{
    any::type_name_of_val,
    collections::{HashMap, HashSet},
    error::Error,
    fmt::{Debug, Display},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    runtime_config::{
        RuntimeConfig,
        effect::EffectComponentConfig,
        input::InputComponentConfig,
        output::OutputComponentConfig,
        shared::{ComponentConfig, Uid},
    },
    types::errors::RuntimeConfigValidationError,
};
use anyhow::{Result, anyhow};

#[derive(PartialEq, Clone)]
pub struct RuntimeConfigValidationHelper {
    pub(crate) name: String,
    pub(crate) type_name: String,
}

pub fn gather_validation_helper_data(
    config: &RuntimeConfig,
) -> HashMap<Uid, RuntimeConfigValidationHelper> {
    let mut map = HashMap::new();
    for config in &config.inputs {
        map.insert(
            config.uid(),
            RuntimeConfigValidationHelper {
                name: config.name(),
                type_name: type_name_of_val(config.config.as_ref()).to_string(),
            },
        );
    }
    for config in &config.effects {
        map.insert(
            config.uid(),
            RuntimeConfigValidationHelper {
                name: config.name(),
                type_name: type_name_of_val(config.config.as_ref()).to_string(),
            },
        );
    }
    for config in &config.outputs {
        map.insert(
            config.uid(),
            RuntimeConfigValidationHelper {
                name: config.name(),
                type_name: type_name_of_val(config.config.as_ref()).to_string(),
            },
        );
    }

    map
}



pub fn ensure_config_bounds<T>(
    some_val: Option<T>,
    lower_bound: T,
    upper_bound: T,
) -> Result<Option<T>>
where
    T: PartialOrd + Debug,
{
    if let Some(v) = some_val {
        if (lower_bound < v) && (v <= upper_bound) {
            Ok(Some(v))
        } else {
            Err(anyhow!(
                "out of bounds [{:?}, {:?}]",
                lower_bound,
                upper_bound
            ))
        }
    } else {
        Ok(None)
    }
}
