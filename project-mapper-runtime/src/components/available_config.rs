use std::collections::HashMap;

use crate::components::marker::{AvailablaeConfig as ConstructAvailablaeConfigFn, Marker};
use project_mapper_core::available_config::config::{
    AvailableConfig, AvailableConfigTrait, AvailableConfigType, AvailableEffectConfig,
    AvailableInputConfig, AvailableOutputConfig,
};

#[derive(PartialEq, Eq, Hash)]
enum LocalCompType {
    Input,
    Effect,
    Output,
}
pub struct AvailableConfigHelper {
    pub result_map: HashMap<(String, LocalCompType), Box<dyn AvailableConfigTrait>>,
    pub recreate_vec: Vec<Marker>,
}

impl AvailableConfigHelper {
    pub fn new() -> Self {
        let mut result_map = HashMap::new();
        let mut recreate_vec = Vec::new();
        for marker in inventory::iter::<Marker> {
            let boxed_config = (marker.available_config)().unwrap();

            let (config, config_type) = match boxed_config.as_ref() {
                AvailableConfigType::Input(config) => (
                    Box::new(config.clone()) as Box<dyn AvailableConfigTrait>,
                    LocalCompType::Input,
                ),
                AvailableConfigType::Effect(config) => (
                    Box::new(config.clone()) as Box<dyn AvailableConfigTrait>,
                    LocalCompType::Effect,
                ),
                AvailableConfigType::Output(config) => (
                    Box::new(config.clone()) as Box<dyn AvailableConfigTrait>,
                    LocalCompType::Output,
                ),
            };

            if config.requires_refresh() {
                recreate_vec.push(marker.clone());
            }
            result_map.insert((marker.name.to_string(), config_type), config);
        }

        Self {
            result_map,
            recreate_vec,
        }
    }

    pub fn get_config(&mut self) -> AvailableConfig {
        for marker in &self.recreate_vec {
            let boxed_config = (marker.available_config)().unwrap();
            match boxed_config.as_ref() {
                AvailableConfigType::Input(config) => {
                    self.result_map.insert(
                        (marker.name.to_string(), LocalCompType::Input),
                        Box::new(config.clone()),
                    );
                }
                AvailableConfigType::Effect(config) => {
                    self.result_map.insert(
                        (marker.name.to_string(), LocalCompType::Effect),
                        Box::new(config.clone()),
                    );
                }
                AvailableConfigType::Output(config) => {
                    self.result_map.insert(
                        (marker.name.to_string(), LocalCompType::Output),
                        Box::new(config.clone()),
                    );
                }
            };
        }

        let mut available_config = AvailableConfig::default();
        for ((name, comp_type), config) in &self.result_map {
            match comp_type {
                LocalCompType::Input => {
                    available_config.inputs.push(
                        config
                            .as_any()
                            .downcast_ref::<AvailableInputConfig>()
                            .unwrap()
                            .clone(),
                    );
                }
                LocalCompType::Effect => {
                    available_config.effects.push(
                        config
                            .as_any()
                            .downcast_ref::<AvailableEffectConfig>()
                            .unwrap()
                            .clone(),
                    );
                }
                LocalCompType::Output => {
                    available_config.outputs.push(
                        config
                            .as_any()
                            .downcast_ref::<AvailableOutputConfig>()
                            .unwrap()
                            .clone(),
                    );
                }
            }
        }

        available_config
    }
}
