use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use project_mapper_core::loader::runtime_loader::export_config_json;
use project_mapper_core::runtime_config::RuntimeConfig;
use project_mapper_core::runtime_config::effect::EffectComponentConfig;
use project_mapper_core::runtime_config::effect::balance::BalanceConfig;
use project_mapper_core::runtime_config::effect::common::DefaultSrcConfig;
use project_mapper_core::runtime_config::effect::fps::FpsConfig;
use project_mapper_core::runtime_config::effect::gamma::GammaConfig;
use project_mapper_core::runtime_config::effect::perspective::PerspectiveConfig;
use project_mapper_core::runtime_config::input::InputComponentConfig;
use project_mapper_core::runtime_config::input::test::TestConfig;
use project_mapper_core::runtime_config::input::uri::UriConfig;
use project_mapper_core::runtime_config::output::OutputComponentConfig;
use project_mapper_core::runtime_config::output::window::{WindowConfig, WindowMode};
use project_mapper_core::types::video::Resolution;
use project_mapper_runtime::components::comp_helper::DefaultComponentHelper;
use project_mapper_runtime::components::factory::DefaultComponentFactory;
use project_mapper_runtime::runtime::runtime::Runtime;
use simple_logger::SimpleLogger;

fn run_main() -> Result<()> {
    //configure logger
    SimpleLogger::new().init().unwrap();

    let _resolution = Resolution {
        width: 1920,
        height: 1080,
    };

    let config = RuntimeConfig {
        inputs: vec![InputComponentConfig {
            uid: 0,
            name: "test_comp".to_string(),
            config: Box::new(TestConfig {fps: 180}),
        }
        ],
        effects: vec![],
        outputs: vec![
            OutputComponentConfig {
                uid: 1,
                name: "output_comp".to_string(),
                config: Box::new(WindowConfig {
                    mode: WindowMode::Windowed {},
                }),
                src_uid: 0,
            },
        ],
    };
    config
        .validate()
        .context("Failed to validate runtime config")?;

    println!(
        "Current json config: '{}'",
        export_config_json(&config).expect("no")
    );

    let runtime = Runtime::new(
        Box::new(DefaultComponentFactory {}),
        Box::new(DefaultComponentHelper::new()),
    )
    .context("Failed to create runtime")?;
    runtime
        .run(Arc::new(Mutex::new(config)))
        .context("Failed to run runtime due to error")
}

fn main() {
    if let Err(error) = run_main() {
        panic!("{:#}", error);
    }
}
