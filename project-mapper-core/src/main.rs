use project_mapper_core::runtime_config::RuntimeConfig;
use project_mapper_core::runtime_config::input::InputComponentConfig;
use project_mapper_core::runtime_config::output::OutputComponentConfig;
use project_mapper_core::runtime_config::output::window::{
    MonitorConfig, WindowConfig, WindowMode,
};
use project_mapper_core::types::video::Resolution;

pub fn main() {
    let resolution = Resolution {
        width: 190,
        height: 100,
    };

    let config = RuntimeConfig {
        inputs: vec![],
        effects: vec![],
        outputs: vec![OutputComponentConfig {
            uid: 1,
            name: "comp_1".to_string(),
            config: Box::new(WindowConfig {
                mode: WindowMode::Exclusive {
                    config: MonitorConfig {
                        name: "monitor_1".to_string(),
                        resolution: resolution,
                        refresh_rate: 10,
                    },
                },
            }),
            src_uid: 0,
        }],
    };

    let serialized_result = serde_json::to_string(&config).expect("no");
    println!("{}", serialized_result);
    let round_trip_res: RuntimeConfig = serde_json::from_str(&serialized_result).unwrap();
    println!("{:?}", round_trip_res);
}
