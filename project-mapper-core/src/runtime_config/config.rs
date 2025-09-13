use std::{
    any::type_name_of_val,
    collections::{HashMap, HashSet},
    error::Error,
    fmt::Display,
};

use serde::{Deserialize, Serialize};

use crate::{
    runtime_config::{
        effect::EffectComponentConfig,
        input::InputComponentConfig,
        output::OutputComponentConfig,
        shared::{ComponentConfig, Uid},
        utils::changes::{RuntimeConfigChangeTracker, gather_config_changes},
        utils::validation::{gather_validation_helper_data},
    },
    types::errors::RuntimeConfigValidationError,
};
use anyhow::Result as AnyhowResult;

// Top-Level Config object for the runtime
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct RuntimeConfig {
    pub inputs: Vec<InputComponentConfig>,
    pub effects: Vec<EffectComponentConfig>,
    pub outputs: Vec<OutputComponentConfig>,
}

impl RuntimeConfig {
    pub fn gather_configs(&self) -> Vec<Box<dyn ComponentConfig>> {
        let mut output_configs = vec![];
        for config in &self.inputs {
            let typed_config: Box<dyn ComponentConfig> = Box::new(config.clone());
            output_configs.push(typed_config)
        }
        for config in &self.effects {
            let typed_config: Box<dyn ComponentConfig> = Box::new(config.clone());
            output_configs.push(typed_config)
        }
        for config in &self.outputs {
            let typed_config: Box<dyn ComponentConfig> = Box::new(config.clone());
            output_configs.push(typed_config)
        }
        output_configs
    }

    pub fn gather_config_changes(
        &self,
        new_config: &RuntimeConfig,
    ) -> AnyhowResult<RuntimeConfigChangeTracker> {
        gather_config_changes(self, new_config)
    }

    // validate if a new config is a valid update of the existing config
    pub fn validate_changes(
        &self,
        new_config: &RuntimeConfig,
    ) -> Result<(), RuntimeConfigValidationError> {
        let current_helper_data = gather_validation_helper_data(self);
        let new_helper_data = gather_validation_helper_data(new_config);

        // first start by generally validating the new config. This ensures no duplicate
        // configs
        new_config.validate()?;

        // ensure that for all current helper data the name/type remains unchanged
        for (uid, helper_data) in current_helper_data.iter() {
            if let Some(new_helper_data) = new_helper_data.get(uid) {
                if helper_data.name != new_helper_data.name {
                    return Err(RuntimeConfigValidationError::from(format!(
                        "Can not change component {}'s name. Previous name: {}",
                        uid, helper_data.name
                    )));
                }
                if helper_data.type_name != new_helper_data.type_name {
                    return Err(RuntimeConfigValidationError::from(format!(
                        "Can not change component {}'s type. Previous type: {}",
                        uid, helper_data.type_name
                    )));
                }
            }
        }

        Ok(())
    }

    // check if a config is valid. Ensures unique names and uids
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

impl PartialEq for RuntimeConfig {
    fn eq(&self, other: &Self) -> bool {
        gather_validation_helper_data(self) == gather_validation_helper_data(other)
    }
}
