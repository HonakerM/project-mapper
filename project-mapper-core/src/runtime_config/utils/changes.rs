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

#[derive(Debug)]
pub struct RuntimeConfigChangeTracker {
    pub updates: Vec<Box<dyn ComponentConfig>>,
    pub deletes: Vec<Uid>,
}

pub fn gather_config_changes(
    org: &RuntimeConfig,
    updated: &RuntimeConfig,
) -> Result<RuntimeConfigChangeTracker> {
    // get a map to later lookup the config for the uid
    let mut new_lookup_helper: HashMap<Uid, Box<dyn ComponentConfig>> = HashMap::new();
    for config in updated.gather_configs() {
        new_lookup_helper.insert(config.uid(), config);
    }

    let mut updates: Vec<Box<dyn ComponentConfig>> = vec![];
    let mut deletes: Vec<Uid> = vec![];

    // track updates for inputs
    gather_config_helper(
        &mut new_lookup_helper,
        &mut updates,
        &mut deletes,
        serde_json::to_value(&org.inputs)?,
        serde_json::to_value(&updated.inputs)?,
    )?;

    // track updates for effects
    gather_config_helper(
        &mut new_lookup_helper,
        &mut updates,
        &mut deletes,
        serde_json::to_value(&org.effects)?,
        serde_json::to_value(&updated.effects)?,
    )?;

    // track updates for outputs
    gather_config_helper(
        &mut new_lookup_helper,
        &mut updates,
        &mut deletes,
        serde_json::to_value(&org.outputs)?,
        serde_json::to_value(&updated.outputs)?,
    )?;

    Ok(RuntimeConfigChangeTracker {
        updates: updates,
        deletes: deletes,
    })
}

fn gather_config_helper(
    lookup_helper: &mut HashMap<Uid, Box<dyn ComponentConfig>>,
    updates: &mut Vec<Box<dyn ComponentConfig>>,
    deletes: &mut Vec<Uid>,
    org: Value,
    updated: Value,
) -> Result<()> {
    if let Value::Array(org_vec) = org
        && let Value::Array(updated_vec) = updated
    {
        let input_helper = compare_config_vecs(org_vec, updated_vec)?;
        println!("Current helper {:?}", input_helper);
        for uid in input_helper.updated {
            let config = lookup_helper.remove(&uid).ok_or(anyhow!(
                "Somehow discovered change in component that doesn't exit"
            ))?;
            updates.push(config)
        }
        deletes.extend(input_helper.deleted);
        Ok(())
    } else {
        Err(anyhow!(
            "Inputs to gather_config_helper do not have the correct type"
        ))
    }
}

#[derive(Debug)]
struct RuntimeConfigChangeHelper {
    pub updated: Vec<Uid>,
    pub deleted: Vec<Uid>,
}
fn compare_config_vecs(
    org_input: Vec<Value>,
    new_input: Vec<Value>,
) -> Result<RuntimeConfigChangeHelper> {
    let mut updated_uids = vec![];
    let mut deleted_uids = vec![];

    // Convert Vec<Value> to HashMap<uid, Value>
    let mut org_map: HashMap<Uid, Value> = HashMap::new();
    for item in org_input {
        org_map.insert(extract_uid_from_value(&item)?, item);
    }

    let mut new_map: HashMap<Uid, Value> = HashMap::new();
    for item in new_input {
        new_map.insert(extract_uid_from_value(&item)?, item);
    }

    for (uid, new_val) in &new_map {
        match org_map.remove(uid) {
            None => updated_uids.push(uid.clone()),
            Some(old_val) if old_val != *new_val => updated_uids.push(uid.clone()),
            Some(_) => {}
        }
    }
    deleted_uids.extend(org_map.keys());

    Ok(RuntimeConfigChangeHelper {
        updated: updated_uids,
        deleted: deleted_uids,
    })
}

fn extract_uid_from_value(value: &Value) -> Result<Uid> {
    let output = value
        .get("uid")
        .ok_or(anyhow!("Unable to find Uid Attribute"))?;
    let output = output
        .as_i64()
        .ok_or(anyhow!("Unable to parse Uid into int"))?;
    Ok(output as Uid)
}
