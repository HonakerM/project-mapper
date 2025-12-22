use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};

use crate::components::marker::{AvailablaeConfig as ConstructAvailablaeConfigFn, Marker};
use project_mapper_core::{
    available_config::{
        config::{AvailableConfig, AvailableConfigTrait, AvailableConfigType},
        effect::AvailableEffectConfig,
        input::AvailableInputConfig,
        output::AvailableOutputConfig,
    },
    types::openapi::OpenAPISchema,
};

const AVAILABLE_CONFIG_REFRESH_DELAY: std::time::Duration = std::time::Duration::from_secs(1);
static AVAILABLE_CONFIG_PROVIDER: LazyLock<Arc<Mutex<AvailableConfigHelper>>> =
    LazyLock::new(|| Arc::new(Mutex::new(AvailableConfigHelper::new())));

pub struct AvailableConfigHelper {
    input_results: HashMap<String, AvailableInputConfig>,
    effect_results: HashMap<String, AvailableEffectConfig>,
    output_results: HashMap<String, AvailableOutputConfig>,
    recreate_vec: Vec<Marker>,

    last_config: AvailableConfig,
    last_update: std::time::Instant,
}

impl AvailableConfigHelper {
    pub fn new() -> Self {
        let mut input_results = HashMap::new();
        let mut effect_results = HashMap::new();
        let mut output_results = HashMap::new();
        let mut recreate_vec = Vec::new();
        for marker in inventory::iter::<Marker> {
            let boxed_config = (marker.available_config)().unwrap();
            let name = marker.name.to_string();

            let required_refresh = match boxed_config.as_ref() {
                AvailableConfigType::Input(config) => {
                    input_results.insert(name.clone(), config.clone());
                    config.requires_refresh()
                }
                AvailableConfigType::Effect(config) => {
                    effect_results.insert(name.clone(), config.clone());
                    config.requires_refresh()
                }
                AvailableConfigType::Output(config) => {
                    output_results.insert(name.clone(), config.clone());
                    config.requires_refresh()
                }
            };

            if required_refresh {
                recreate_vec.push(marker.clone());
            }
        }

        let mut val = Self {
            input_results,
            effect_results,
            output_results,
            recreate_vec,
            last_config: AvailableConfig::default(),
            last_update: std::time::Instant::now(),
        };
        val.gather_config();

        val
    }

    pub fn get_config() -> AvailableConfig {
        let mut guard = AVAILABLE_CONFIG_PROVIDER.lock().unwrap();
        if guard.last_update.elapsed() > AVAILABLE_CONFIG_REFRESH_DELAY {
            guard.gather_config();
        };
        guard.last_config.clone()
    }

    fn gather_config(&mut self) -> AvailableConfig {
        for marker in &self.recreate_vec {
            let boxed_config = (marker.available_config)().unwrap();
            match boxed_config.as_ref() {
                AvailableConfigType::Input(config) => {
                    self.input_results
                        .insert((marker.name.to_string()), config.clone());
                }
                AvailableConfigType::Effect(config) => {
                    self.effect_results
                        .insert((marker.name.to_string()), config.clone());
                }
                AvailableConfigType::Output(config) => {
                    self.output_results
                        .insert((marker.name.to_string()), config.clone());
                }
            };
        }

        let mut available_config = AvailableConfig::default();
        for config in self.input_results.values() {
            available_config.inputs.push(config.clone());
        }
        for config in self.effect_results.values() {
            available_config.effects.push(config.clone());
        }
        for config in self.output_results.values() {
            available_config.outputs.push(config.clone());
        }

        self.last_config = available_config.clone();
        self.last_update = std::time::Instant::now();

        available_config
    }
}
