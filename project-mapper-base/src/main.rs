use project_mapper_base::{effect::balance::BalanceConfig, prelude::*};
use project_mapper_core::runtime_config::effect::common::EffectConfigTrait;

fn main() {
    if let Err(err) = project_mapper_base::entrypoint::run_main() {
        panic!("{}", err)
    };
}
