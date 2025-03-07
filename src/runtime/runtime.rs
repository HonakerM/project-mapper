//! This example demonstrates how to output GL textures, within an EGL/X11 context provided by the
//! application, and render those textures in the GL application.
//!
//! This example follow common patterns from `glutin`:
//! <https://github.com/rust-windowing/glutin/blob/master/glutin_examples/src/lib.rs>

// {videotestsrc} - { glsinkbin }

use std::{
    ffi::{CStr, CString},
    mem,
    num::NonZeroU32,
    ptr,
};

#[path = "../pipeline/pipeline.rs"]
mod pipeline;


use anyhow::{Context, Result};
use glutin::{
    config::GetGlConfig as _,
    context::AsRawContext as _,
    display::{AsRawDisplay as _, GetGlDisplay as _},
    prelude::*,
};
use glutin_winit::GlWindow as _;
use gst::{element_error, PadProbeReturn, PadProbeType, QueryViewMut};
use gst_gl::prelude::*;
use raw_window_handle::HasWindowHandle as _;
use winit::dpi::{LogicalSize, PhysicalPosition, PhysicalSize};


#[derive(Debug)]
pub(crate) enum Message {
    Frame(gst_video::VideoInfo, gst::Buffer),
    BusMessage(gst::Message),
}

pub(crate) struct Runtime {
    pub pipeline: gst::Pipeline,
}

impl Runtime {
    pub(crate) fn new(gl_element: Option<&gst::Element> ) -> Result<Runtime> {
        gst::init()?;

        let media_pipeline = pipeline::MediaPipeline::new(None)?;
 
        let runtime = Runtime {
            pipeline,
        };

        app.setup()?;
        Ok(app)
    }


    pub fn run(mut self) -> Result<()> {

    }
}
