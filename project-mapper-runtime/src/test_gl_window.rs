

use anyhow::{Result};
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowAttributes};
use winit::application::ApplicationHandler;
use std::error::Error;
use std::ffi::{CStr, CString};
use std::num::NonZeroU32;
use std::ops::Deref;
use glutin::display::AsRawDisplay;
use glutin::context::AsRawContext;
use gst_gl::{GLPlatform, GLVideoFrameExt};
use crate::opengl::{self, gl};

use raw_window_handle::HasWindowHandle;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, NamedKey};

use glutin::config::{Config, ConfigTemplateBuilder, GetGlConfig};
use glutin::context::{
    ContextApi, ContextAttributesBuilder, NotCurrentContext, PossiblyCurrentContext, Version,
};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{Surface, SwapInterval, WindowSurface};
use winit::window::{WindowId};


use glutin_winit::{DisplayBuilder, GlWindow};


pub fn test_main() -> Result<()> {
    // init gst to not break things
    gst::init()?;

    // Winit Init Stuff
    let event_loop: winit::event_loop::EventLoop<Message> =
    winit::event_loop::EventLoop::with_user_event().build()?;

    let mut app = App::new(&event_loop);
    event_loop.run_app(&mut app);

    Ok(())
}


#[derive(Debug)]
pub(crate) enum Message {
    Frame(gst_video::VideoInfo, gst::Buffer, WindowId),
    BusMessage(gst::Message),
}


struct AppState {
    gst_context: gst_gl::GLContext,
    gl_context: NotCurrentContext,
    gl_surface: Surface<WindowSurface>,
    // NOTE: Window should be dropped after all resources created using its
    // raw-window-handle.
    window: Window,
}


struct App {
    state: AppState,
}

impl App {
    fn new(event_loop: &winit::event_loop::EventLoop<Message>) -> Self {
        Self {
            state: App::create_window(glib::GString::from("Test"), event_loop).expect("uh oh"),
        }
    }

    fn create_window(name: glib::GString, event_loop: &winit::event_loop::EventLoop<Message>) -> Result<AppState> {
        // Create the winit window builder and template
        let window_attributes = winit::window::Window::default_attributes()
        .with_transparent(true)
        .with_title(name.clone().to_string());
        let display_builder = glutin_winit::DisplayBuilder::new().with_window_attributes(Some(window_attributes));
        let template = glutin::config::ConfigTemplateBuilder::new().with_alpha_size(8).with_transparency(cfg!(cgl_backend));

        // Create the winit window and get the gl config
        let build_result = display_builder.clone().build(
            event_loop,
            template,
            gl_config_picker,
        );
        let (window, gl_config) = build_result.expect("Failed to build display");
        let window = window.expect("give me a window");

        // Create the OpenGL Context
        let gl_context = gl_create_context(&window, &gl_config);

        // Create the OpenGL Surface from the context
        let attrs = window
        .build_surface_attributes(Default::default())
        .expect("Failed to build surface attributes");
        let gl_surface =
            unsafe { gl_config.display().create_window_surface(&gl_config, &attrs).unwrap() };


        let gst_context = gst_create_context(&gl_config, &gl_context).unwrap();
        // Set the current state
        Ok(AppState {
            window: window,
            gl_context: gl_context,
            gl_surface: gl_surface, 
            gst_context: gst_context,
        })
    }
}

impl ApplicationHandler<Message> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
    }
}


// Find the config with the maximum number of samples, so our triangle will be
// smooth.
pub fn gl_config_picker(configs: Box<dyn Iterator<Item = Config> + '_>) -> Config {
    configs
        .reduce(|accum, config| {
            let transparency_check = config.supports_transparency().unwrap_or(false)
                & !accum.supports_transparency().unwrap_or(false);

            if transparency_check || config.num_samples() > accum.num_samples() {
                config
            } else {
                accum
            }
        })
        .unwrap()
}


fn gst_create_context( gl_config: &Config, context: &NotCurrentContext) -> Result<gst_gl::GLContext> {
    // Get the current api
    let api = opengl::map_gl_api(gl_config.api());

    // get the raw display from the config
    let gl_display = gl_config.display();
    let raw_gl_display = gl_display.raw_display();
    
    // get the raw context
    let raw_gl_context = context.raw_context();

    let (raw_gl_context, gst_gl_display, platform): (usize, gst_gl::GLDisplay, GLPlatform) =
        match (raw_gl_display, raw_gl_context) {
            #[cfg(feature = "gst-gl-egl")]
            (
                glutin::display::RawDisplay::Egl(egl_display),
                glutin::context::RawContext::Egl(egl_context),
            ) => {
                let gl_display =
                    unsafe { gst_gl_egl::GLDisplayEGL::with_egl_display(egl_display as usize) }
                        .context("Failed to create GLDisplayEGL from raw `EGLDisplay`")?
                        .upcast::<gst_gl::GLDisplay>();
                (egl_context as usize, gl_display, gst_gl::GLPlatform::EGL)
            }
            #[cfg(feature = "gst-gl-x11")]
            (
                glutin::display::RawDisplay::Glx(glx_display),
                glutin::context::RawContext::Glx(glx_context),
            ) => {
                let gl_display =
                    unsafe { gst_gl_x11::GLDisplayX11::with_display(glx_display as usize) }
                        .context("Failed to create GLDisplayX11 from raw X11 `Display`")?
                        .upcast::<gst_gl::GLDisplay>();
                (glx_context as usize, gl_display, gst_gl::GLPlatform::GLX)
            }
            #[cfg(windows)]
            (
                glutin::display::RawDisplay::Wgl,
                glutin::context::RawContext::Wgl(wgl_context),
            ) => {
                let gl_display = gst_gl::GLDisplay::new();
                (wgl_context as usize, gl_display, gst_gl::GLPlatform::WGL)
            }
            #[allow(unreachable_patterns)]
            handler => anyhow::bail!("Unsupported platform: {handler:?}."),
        };
        
    let glutin_context = unsafe {
        gst_gl::GLContext::new_wrapped(&gst_gl_display, raw_gl_context, platform, api)
    }.unwrap();

    Ok(glutin_context)
}


fn gl_create_context(window: &Window, gl_config: &Config) -> NotCurrentContext {
    let raw_window_handle = window.window_handle().ok().map(|wh| wh.as_raw());

    // The context creation part.
    let context_attributes = ContextAttributesBuilder::new().build(raw_window_handle);

    // Since glutin by default tries to create OpenGL core context, which may not be
    // present we should try gles.
    let fallback_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::Gles(None))
        .build(raw_window_handle);

    // There are also some old devices that support neither modern OpenGL nor GLES.
    // To support these we can try and create a 2.1 context.
    let legacy_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::OpenGl(Some(Version::new(2, 1))))
        .build(raw_window_handle);

    // Reuse the uncurrented context from a suspended() call if it exists, otherwise
    // this is the first time resumed() is called, where the context still
    // has to be created.
    let gl_display = gl_config.display();

    unsafe {
        gl_display.create_context(gl_config, &context_attributes).unwrap_or_else(|_| {
            gl_display.create_context(gl_config, &fallback_context_attributes).unwrap_or_else(
                |_| {
                    gl_display
                        .create_context(gl_config, &legacy_context_attributes)
                        .expect("failed to create context")
                },
            )
        })
    }
}