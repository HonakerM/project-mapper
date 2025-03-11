// This example demonstrates GStreamer's playbin element.
// This element takes an arbitrary URI as parameter, and if there is a source
// element within gstreamer, that supports this uri, the playbin will try
// to automatically create a pipeline that properly plays this media source.
// For this, the playbin internally relies on more bin elements, like the
// autovideosink and the decodebin.
// Essentially, this element is a single-element pipeline able to play
// any format from any uri-addressable source that gstreamer supports.
// Much of the playbin's behavior can be controlled by so-called flags, as well
// as the playbin's properties and signals.

use anyhow::Result;
use api::cli::Cli;
use clap::Parser;
use gst_gl::prelude::*;
use std::{env, fs};

#[path = "../runtime/runtime.rs"]
mod runtime;

#[path = "../window/handler.rs"]
mod window_handler;

#[path = "../render/opengl.rs"]
mod opengl;

#[path = "../pipeline/pipeline.rs"]
mod pipeline;

#[path = "../api/mod.rs"]
mod api;

#[path = "../utils/main_wrapper.rs"]
pub mod main_wrapper;

fn entrypoint() -> Result<()> {
    let args = Cli::parse();
    match &args {
        Cli::Run(run) => run.run(),
        Cli::GetAvailableConfig(gac) => gac.run(),
    }
}

fn main() -> Result<()> {
    // examples_common::run is only required to set up the application environment on macOS
    // (but not necessary in normal Cocoa applications where this is set up automatically)
    main_wrapper::run(entrypoint)
}
