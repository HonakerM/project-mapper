use project_mapper_core::config::Config;
use project_mapper_core::config::output::OutputComponentConfig;
use project_mapper_core::config::output::common::OutputConfig;
use project_mapper_core::config::output::window::{MonitorConfig, WindowConfig, WindowMode};
use project_mapper_core::types::video::Resolution;

pub fn main() {
    let resolution = Resolution {
        width: 190,
        height: 100,
    };

    let config = Config {
        outputs: vec![OutputComponentConfig {
            uid: 0,
            name: "comp_1".to_string(),
            config: OutputConfig::Window(WindowConfig {
                mode: WindowMode::Exclusive {
                    config: MonitorConfig {
                        name: "monitor_1".to_string(),
                        resolution: resolution,
                        refresh_rate: 10,
                    },
                },
            }),
        }],
    };

    let serialized_result = serde_json::to_string(&config).expect("no");
    println!("{}", serialized_result);
    let round_trip_res: Config = serde_json::from_str(&serialized_result).unwrap();
    println!("{:?}", round_trip_res);
}
