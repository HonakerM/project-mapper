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
        shared::{ComponentConfig, UID_MAX, UID_MIN, Uid, uid_openapi_schema},
        utils::{changes::RuntimeConfigChangeTracker, validation::gather_validation_helper_data},
    },
    types::{errors::RuntimeConfigValidationError, openapi::OpenAPISchema},
};
use anyhow::Result as AnyhowResult;

const OPENAPI_PROPERTIES_KEY: &str = "properties";
const OPENAPI_REQUIRED_KEY: &str = "required";

pub fn default_type_schema(type_name: String) -> serde_json::Value {
    serde_json::json!({"type":"string","const":type_name,"description":"The static identifier for this component type"})
}

pub fn insert_type_into_config(ac: &mut serde_json::Value, type_name: String) {
    match ac.as_object_mut() {
        Some(ac_obj) => {
            match ac_obj.get_mut(OPENAPI_PROPERTIES_KEY) {
                Some(params) => match params.as_object_mut() {
                    Some(map) => {
                        map.insert("type".to_string(), default_type_schema(type_name));
                    }
                    None => {}
                },
                None => {
                    ac_obj.insert(
                        OPENAPI_PROPERTIES_KEY.to_string(),
                        default_type_schema(type_name),
                    );
                }
            };
            match ac_obj.get_mut(OPENAPI_REQUIRED_KEY) {
                Some(params) => match params.as_array_mut() {
                    Some(arr) => {
                        arr.push(serde_json::Value::String("type".to_string()));
                    }
                    None => {}
                },
                None => {}
            };
        }
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
            "id": uid_openapi_schema(),
        },
        "required": ["name","id"],
        "additionalProperties": false,
    })
    .try_into()
    .unwrap()
}

pub fn insert_config_into_base(
    base: &mut serde_json::Value,
    config_name: String,
    ac: serde_json::Value,
) {
    match base.as_object_mut() {
        Some(base_obj) => {
            match base_obj.get_mut(OPENAPI_PROPERTIES_KEY) {
                Some(params) => match params.as_object_mut() {
                    Some(map) => {
                        map.insert(config_name.clone(), ac);
                    }
                    None => {}
                },
                None => {}
            };
            match base_obj.get_mut(OPENAPI_REQUIRED_KEY) {
                Some(params) => match params.as_array_mut() {
                    Some(arr) => {
                        arr.push(serde_json::Value::String(config_name));
                    }
                    None => {}
                },
                None => {}
            }
        }
        None => {}
    }
}
