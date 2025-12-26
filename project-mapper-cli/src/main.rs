#[path = "./entrypoint.rs"]
pub mod entrypoint;

use log::{error, info};

use project_mapper_runtime::components::available_config::AvailableConfigHelper;

fn main() {
    let config = AvailableConfigHelper::get_config();
    info!(
        "Available Config:\n{}",
        serde_json::to_string_pretty(&config).unwrap()
    );
    info!(
        "Available Config Schema:\n{}",
        serde_json::to_string_pretty(&config.get_schema()).unwrap()
    );

    if let Err(err) = entrypoint::run_main() {
        error!("{:?}", err);
        panic!("{}", err)
    };
}
