use project_mapper_base::{effect::balance::BalanceConfig, prelude::*};
use project_mapper_core::runtime_config::effect::common::EffectConfigTrait;
use project_mapper_runtime::components::available_config::AvailableConfigHelper;

fn main() {
    let config = AvailableConfigHelper::get_config();
    println!(
        "Available Config:\n{}",
        serde_json::to_string_pretty(&config).unwrap()
    );

    // if let Err(err) = project_mapper_base::entrypoint::run_main() {
    //     panic!("{}", err)
    // };
}
