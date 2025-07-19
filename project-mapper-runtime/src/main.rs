use anyhow::Result;
use project_mapper_core::runtime_config::RuntimeConfig;
use project_mapper_core::runtime_config::input::InputComponentConfig;
use project_mapper_core::runtime_config::input::common::InputConfig;
use project_mapper_core::runtime_config::input::test::TestConfig;
use project_mapper_core::runtime_config::output::OutputComponentConfig;
use project_mapper_core::runtime_config::output::common::OutputConfig;
use project_mapper_core::runtime_config::output::window::{
    MonitorConfig, WindowConfig, WindowMode,
};
use project_mapper_core::types::video::Resolution;
use project_mapper_runtime::components::comp_helper::ComponentHelper;
use project_mapper_runtime::runtime::runtime::Runtime;

fn run_main() -> Result<()> {
    let resolution = Resolution {
        width: 1920,
        height: 1080,
    };

    let config = RuntimeConfig {
        inputs: vec![InputComponentConfig {
            uid: 0,
            name: "in_comp".to_string(),
            config: InputConfig::Test(TestConfig {}),
        }],
        effects: vec![],
        outputs: vec![
            OutputComponentConfig {
                uid: 1,
                name: "comp_1".to_string(),
                config: OutputConfig::Window(WindowConfig {
                    mode: WindowMode::Borderless {
                        name: "\\\\.\\DISPLAY1".to_string(),
                    },
                }),
                src_uid: 0,
            }, /*,
               OutputComponentConfig {
                   uid: 2,
                   name: "comp_2".to_string(),
                   config: OutputConfig::Window(WindowConfig {
                       mode: WindowMode::Exclusive {
                           config: MonitorConfig {
                               name: ".DISPLAY1".to_string(),
                               resolution: resolution.clone(),
                               refresh_rate: 60000,
                           },
                       },
                   }),
                   src_uid: 0,
               },*/
            OutputComponentConfig {
                uid: 3,
                name: "comp_3".to_string(),
                config: OutputConfig::Window(WindowConfig {
                    mode: WindowMode::Windowed {},
                }),
                src_uid: 0,
            },
        ],
    };

    let runtime = Runtime::new(config, Box::new(ComponentHelper::new()))?;
    runtime.run()
}

fn main() {
    if let Err(error) = run_main() {
        panic!("{}", error);
    }
}
