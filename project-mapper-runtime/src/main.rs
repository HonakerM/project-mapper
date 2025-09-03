use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use gst::prelude::{ElementExt, ElementExtManual, GstBinExtManual};
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

    let mut config = RuntimeConfig {
        inputs: vec![InputComponentConfig {
            uid: 0,
            name: "test_comp".to_string(),
            config: Box::new(TestConfig {fps: 180}),
        },InputComponentConfig {
            uid: 4,
            name: "uri_comp".to_string(),
            config: Box::new(UriConfig {uri: "http://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4".to_string()}),
        },
        ],
        effects: vec![
            EffectComponentConfig {
                uid: 6,
                name: "super_bright".to_string(),
                config: Box::new(BalanceConfig {
                    brightness: Some(1.0),
                    contrast: Some(1.0),
                    saturation: None,
                    hue: None,
                }),
                srcs: vec![Box::new(DefaultSrcConfig {
                    uid: 0
                })],            },
            EffectComponentConfig {
                uid: 7,
                name: "gamma_bright".to_string(),
                config: Box::new(GammaConfig {
                    gamma: Some(0.1)
                }),
                srcs: vec![Box::new(DefaultSrcConfig {
                    uid: 0
                })],
            },            EffectComponentConfig {
                uid: 71,
                name: "fps_in_1".to_string(),
                config: Box::new(FpsConfig { max_rate: Some(60) }),
                srcs: vec![Box::new(DefaultSrcConfig {
                    uid: 7
                })],
            },            EffectComponentConfig {
                uid: 72,
                name: "fps_in_2".to_string(),
                config: Box::new(FpsConfig { max_rate: Some(60) }),
                srcs: vec![Box::new(DefaultSrcConfig {
                    uid: 8
                })],
            },
            EffectComponentConfig {
                uid: 8,
                name: "perspective_transform".to_string(),
                config: Box::new(PerspectiveConfig {
                    matrix: [
                        1.0, 0.0, 0.0, // First row
                        0.0, 1.0, 0.0, // Second row
                        0.0, 0.0, 1.0, // Third row
                    ],
                }),
                srcs: vec![Box::new(DefaultSrcConfig {
                    uid: 0,
                })],
            },
        ],
        outputs: vec![ 
            /* 
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
                config: Box::new(WindowConfig {
                    mode: WindowMode::Windowed {},
                }),
                src_uid: 72,
            },            OutputComponentConfig {
                uid: 12,
                name: "comp_5".to_string(),
                config: Box::new(WindowConfig {
                    mode: WindowMode::Windowed {},
                }),
                src_uid: 72,
            },
        ],
    };

     #[cfg(target_os = "macos")]
     config.outputs.push(
        OutputComponentConfig {
            uid: 1,
            name: "comp_1".to_string(),
            config: Box::new(WindowConfig {
                mode: WindowMode::Borderless {
                    name: "Monitor #41022".to_string(),
                },
            }),
            src_uid: 71,
        });
    #[cfg(target_os = "windows")]
     config.outputs.push(
        OutputComponentConfig {
            uid: 1,
            name: "comp_1".to_string(),
            config: Box::new(WindowConfig {
                mode: WindowMode::Borderless {
                    name: "\\\\.\\DISPLAY1".to_string(),
                },
            }),
            src_uid: 71,
        }
     );

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


fn test_main() -> Result<()> {
    gst::init()?;

    let pipeline = gst::Pipeline::new();

    // construct test gstreamer element
    let test_element = gst::ElementFactory::make("videotestsrc")
        .name("test-in")
        .build()?;
    let effect_element = gst::ElementFactory::make("twirl")
        .name("test-effect")
        .build()?;
    let output = gst::ElementFactory::make("fakesink")
        .name("test-output")
        .build()?;

    pipeline.add_many([&test_element, &effect_element, &output])?;
    test_element.sync_state_with_parent()?;
    effect_element.sync_state_with_parent()?;
    output.sync_state_with_parent()?;
    test_element.link(&effect_element)?;
    effect_element.link(&output)?;

    
    // Start the pipeline
    pipeline
        .set_state(gst::State::Playing)
        .context("Failed to start pipeline")?;

    loop {
        thread::sleep(Duration::from_secs(2));

        effect_element.unlink(&output);
        test_element.unlink(&effect_element);
        test_element.link(&output)?;

        thread::sleep(Duration::from_secs(2));
        test_element.unlink(&output);
        effect_element.link(&output)?;
        test_element.link(&effect_element)?;
    }
    
    Ok(())

}

fn main() {
    if let Err(error) = test_main() {
        panic!("{:#}", error);
    }
}
