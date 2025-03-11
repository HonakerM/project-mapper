use std::sync::mpsc;

use glutin::api::egl::config;
use project_mapper_core::config::{
    events::OptionEvent,
    options::{AvailableConfig, RegionTypeOptions, SinkTypeOptions, SourceTypeOptions},
};
use raw_window_handle::HasDisplayHandle;

use crate::window_handler::{self, config::ConfigHandler};

use anyhow::Result;

pub fn generate_options() -> Result<AvailableConfig> {
    gst::init()?;

    let opengl_sink = generate_opengl_option()?;
    let uri_source = generate_uri_option()?;
    let test_source = SourceTypeOptions::Test {};
    let display_region = RegionTypeOptions::Display {};

    Ok(AvailableConfig {
        sinks: vec![opengl_sink],
        sources: vec![uri_source, test_source],
        regions: vec![display_region],
    })
}

pub fn generate_opengl_option() -> Result<SinkTypeOptions> {
    let event_loop: winit::event_loop::EventLoop<window_handler::Message> =
        winit::event_loop::EventLoop::with_user_event().build()?;

    let (send, recv) = mpsc::channel();

    let mut config_generator = ConfigHandler::new(send.clone());

    // run event loop until completion
    event_loop.run_app(&mut config_generator)?;

    // get and parse configs. Do simple for now
    let event = recv.recv()?;

    match event {
        OptionEvent::OpenGLWindowOptions(options) => Ok(options),
    }
}

pub fn generate_uri_option() -> Result<SourceTypeOptions> {
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

    Ok(SourceTypeOptions::URI {
        uri_types: uri_types,
    })
}
