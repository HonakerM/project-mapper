use std::sync::mpsc;

use glutin::api::egl::config;
use project_mapper_core::config::{
    events::OptionEvent,
    options::{
        AvailableConfig, RegionOption, RegionTypeOptions, SinkOption, SinkTypeOptions,
        SourceOption, SourceTypeOptions,
    },
};
use raw_window_handle::HasDisplayHandle;

use crate::window_handler::{self, config::ConfigHandler};

use anyhow::Result;

pub fn generate_options() -> Result<AvailableConfig> {
    gst::init()?;

    let opengl_sink = generate_opengl_option()?;
    let uri_source = generate_uri_option()?;
    let test_source = SourceOption {
        type_name: "Test".to_owned(),
        type_options: SourceTypeOptions::Test {},
    };
    let display_region = RegionOption {
        type_name: "Display".to_owned(),
        type_options: RegionTypeOptions::Display {},
    };
    Ok(AvailableConfig {
        sinks: vec![opengl_sink],
        sources: vec![uri_source, test_source],
        regions: vec![display_region],
    })
}

pub fn generate_opengl_option() -> Result<SinkOption> {
    let event_loop: winit::event_loop::EventLoop<window_handler::Message> =
        winit::event_loop::EventLoop::with_user_event().build()?;

    let (send, recv) = mpsc::channel();

    let mut config_generator = ConfigHandler::new(send.clone());

    // run event loop until completion
    event_loop.run_app(&mut config_generator)?;

    // get and parse configs. Do simple for now
    let event = recv.recv()?;

    match event {
        OptionEvent::OpenGLWindowOptions(full_screen) => Ok(SinkOption {
            type_name: String::from("OpenGLWindow"),
            type_options: SinkTypeOptions::OpenGLWindow {
                full_screen: full_screen,
            },
        }),
    }
}

pub fn generate_uri_option() -> Result<SourceOption> {
    let mut uri_types = vec![];

    let factory_types = vec![
        gst::ElementFactoryType::SRC,
        gst::ElementFactoryType::MEDIA_VIDEO,
    ];
    for factory_type in factory_types {
        let element_factories =
            gst::ElementFactory::factories_with_type(factory_type, gst::Rank::NONE);
        for factory in element_factories {
            let factory_protocols = factory.uri_protocols();
            for protocol in factory_protocols {
                uri_types.push(protocol.to_string());
            }
        }
    }

    Ok(SourceOption {
        type_name: String::from("URI"),
        type_options: SourceTypeOptions::URI {
            uri_types: uri_types,
        },
    })
}
