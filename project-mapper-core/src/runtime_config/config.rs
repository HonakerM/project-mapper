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

#[derive(PartialEq, Clone)]
struct RuntimeConfigValidationHelper {
    name: String,
    type_name: String,
}

impl RuntimeConfig {
    // validate if a new config is a valid update of the existing config
    pub fn validate_changes(
        &self,
        new_config: &RuntimeConfig,
    ) -> Result<(), RuntimeConfigValidationError> {
        let current_helper_data = self.gather_validation_helper_data();
        let new_helper_data = new_config.gather_validation_helper_data();

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

    fn gather_validation_helper_data(&self) -> HashMap<Uid, RuntimeConfigValidationHelper> {
        let mut map = HashMap::new();
        for config in &self.inputs {
            map.insert(
                config.uid(),
                RuntimeConfigValidationHelper {
                    name: config.name(),
                    type_name: type_name_of_val(config.config.as_ref()).to_string(),
                },
            );
        }
        for config in &self.effects {
            map.insert(
                config.uid(),
                RuntimeConfigValidationHelper {
                    name: config.name(),
                    type_name: type_name_of_val(config.config.as_ref()).to_string(),
                },
            );
        }
        for config in &self.outputs {
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
}

impl PartialEq for RuntimeConfig {
    fn eq(&self, other: &Self) -> bool {
        self.gather_validation_helper_data() == other.gather_validation_helper_data()
    }
}
