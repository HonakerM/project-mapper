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
        shared::{ComponentConfig, UID_MAX, UID_MIN, Uid},
        utils::{changes::RuntimeConfigChangeTracker, validation::gather_validation_helper_data},
    },
    types::{errors::RuntimeConfigValidationError, openapi::OpenAPISchema},
};
use anyhow::Result as AnyhowResult;

pub fn insert_type_into_config(ac: &mut serde_json::Value, type_name: String) {
    match ac.as_object_mut() {
        Some(ac_obj) => match ac_obj.get_mut("parameters") {
            Some(params) => match params.as_object_mut() {
                Some(map) => {
                    map.insert(
                                "type".to_string(),
                                serde_json::json!({"type":"string","enum":[type_name],"description":"The static identifier for this component type"}),
                            );
                }
                None => {}
            },
            None => {}
        },
        None => {}
    }
}

pub fn construct_base_schema() -> OpenAPISchema {
    serde_json::json!({
        "type": "object",
        "properties": {
            "name": {
                "type": "string",
                "description": "The name of this component instance"
            },
            "id": {
                "type": "integer",
                "format":"int32",
                "description": "The unique identifier for this component instance.",
                "minimum":UID_MIN,
                "maximum":UID_MAX,
            }
        },
        "required": [],
        "additionalProperties": false,
    })
    .try_into()
    .unwrap()
}

pub fn insert_config_into_base(
    base: &mut serde_json::Value,
    ac: serde_json::Value,
    config_name: String,
) {
    match base.as_object_mut() {
        Some(base_obj) => match base_obj.get_mut("parameters") {
            Some(params) => match params.as_object_mut() {
                Some(map) => {
                    map.insert(config_name, ac);
                }
                None => {}
            },
            None => {}
        },
        None => {}
    }
}
