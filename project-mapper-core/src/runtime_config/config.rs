use std::{collections::HashSet, error::Error, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::{
    runtime_config::{
        effect::EffectComponentConfig,
        input::InputComponentConfig,
        output::OutputComponentConfig,
        shared::{ComponentConfig, Uid},
    },
    types::errors::RuntimeConfigValidationError,
};

// Top-Level Config object for the runtime
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RuntimeConfig {
    pub inputs: Vec<InputComponentConfig>,
    pub effects: Vec<EffectComponentConfig>,
    pub outputs: Vec<OutputComponentConfig>,
}

impl RuntimeConfig {
    pub fn validate(&self) -> Result<(), RuntimeConfigValidationError> {
        let mut name_set: HashSet<String> = HashSet::new();
        let mut uid_set: HashSet<Uid> = HashSet::new();

        for component in &self.inputs {
            RuntimeConfig::validate_config(
                component.name(),
                component.uid(),
                &mut name_set,
                &mut uid_set,
            )?;
        }
        for component in &self.effects {
            RuntimeConfig::validate_config(
                component.name(),
                component.uid(),
                &mut name_set,
                &mut uid_set,
            )?;
        }
        for component in &self.outputs {
            RuntimeConfig::validate_config(
                component.name(),
                component.uid(),
                &mut name_set,
                &mut uid_set,
            )?;
        }
        Ok(())
    }

    fn validate_config(
        name: String,
        uid: Uid,
        name_set: &mut HashSet<String>,
        uid_set: &mut HashSet<Uid>,
    ) -> Result<(), RuntimeConfigValidationError> {
        if name_set.contains(&name) {
            return Err(RuntimeConfigValidationError::from(format!(
                "Duplicate Name in config: {}",
                name
            )));
        }
        name_set.insert(name);

        // ensure all uids are positive. negative uids have special meaning
        if uid < 0 {
            return Err(RuntimeConfigValidationError::from(format!(
                "Uid must be postive: {}",
                uid
            )));
        }

        if uid_set.contains(&uid) {
            return Err(RuntimeConfigValidationError::from(format!(
                "Duplicate Uid in config: {}",
                uid
            )));
        }
        uid_set.insert(uid);

        Ok(())
    }
}
