use std::collections::HashMap;
use std::fs;
use std::sync::mpsc;

use crate::config::events;
use crate::opengl::{self, gl};
use anyhow::{Context, Result};
use glutin::config::{GetGlConfig, GlConfig};
use glutin::context::AsRawContext;
use glutin::display::{AsRawDisplay, GetGlDisplay};
use glutin::prelude::{GlDisplay, NotCurrentGlContext};
use glutin::surface::GlSurface;
use glutin_winit::GlWindow;
use gst::prelude::GstObjectExt;
use gst::{PadProbeReturn, PadProbeType, QueryViewMut, element_error};
use gst_gl::GLVideoFrameExt;
use gst_gl::prelude::GLContextExt;
use gst_video::VideoFrameExt;
use raw_window_handle::HasWindowHandle;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

struct WindowData {
    window: Window,
    running_state: Option<(
        opengl::Gl,
        glutin::context::PossiblyCurrentContext,
        glutin::surface::Surface<glutin::surface::WindowSurface>,
    )>,
    not_current_gl_context: Option<glutin::context::NotCurrentContext>,
    glutin_context: gst_gl::GLContext,
}

impl WindowData {
    /// Should be called from within the event loop
    fn redraw(&self, current_frame: gst_gl::GLVideoFrame<gst_gl::gl_video_frame::Readable>) {
        if let Some((gl, gl_context, gl_surface)) = &self.running_state {
            let sync_meta = current_frame.buffer().meta::<gst_gl::GLSyncMeta>().unwrap();
            sync_meta.wait(&self.glutin_context);
            if let Ok(texture) = current_frame.texture_id(0) {
                gl.draw_frame(texture as gl::types::GLuint);
            }
            gl_surface.swap_buffers(gl_context).unwrap();
        }
    }
}

#[derive(Clone)]
struct AppSinkData {
    window_id: Option<WindowId>,
    config: crate::config::sink::SinkConfig,
}

#[derive(Debug)]
pub(crate) enum Message {
    Frame(gst_video::VideoInfo, gst::Buffer, glib::GString),
    BusMessage(gst::Message),
}

pub struct WindowHandler {
    sink_mapping: HashMap<glib::GString, AppSinkData>,
    windows: HashMap<WindowId, WindowData>,
    event_proxy: winit::event_loop::EventLoopProxy<Message>,
    event_sender: mpsc::Sender<events::RuntimeEvent>,
}

impl WindowHandler {
    pub fn new(
        event_proxy: winit::event_loop::EventLoopProxy<Message>,
        event_sender: mpsc::Sender<events::RuntimeEvent>,
    ) -> WindowHandler {
        WindowHandler {
            sink_mapping: HashMap::new(),
            windows: HashMap::new(),
            event_proxy: event_proxy,
            event_sender: event_sender,
        }
    }

    pub fn add_sink(&mut self, appsink: gst_app::AppSink, config: crate::config::sink::SinkConfig) {
        let event_proxy = self.event_proxy.clone();
        appsink.set_callbacks(
            gst_app::AppSinkCallbacks::builder()
                .new_sample(move |appsink| {
                    let sample = appsink.pull_sample().map_err(|_| gst::FlowError::Eos)?;
                    let info = sample
                        .caps()
                        .and_then(|caps| gst_video::VideoInfo::from_caps(caps).ok())
                        .ok_or_else(|| {
                            element_error!(
                                appsink,
                                gst::ResourceError::Failed,
                                ("Failed to get video info from sample")
                            );
                            gst::FlowError::NotNegotiated
                        })?;
                    let mut buffer = sample.buffer_owned().unwrap();
                    {
                        let context = match (buffer.n_memory() > 0)
                            .then(|| buffer.peek_memory(0))
                            .and_then(|m| m.downcast_memory_ref::<gst_gl::GLBaseMemory>())
                            .map(|m| m.context())
                        {
                            Some(context) => context.clone(),
                            None => {
                                element_error!(
                                    appsink,
                                    gst::ResourceError::Failed,
                                    ("Failed to get GL context from buffer")
                                );
                                return Err(gst::FlowError::Error);
                            }
                        };
                        if let Some(meta) = buffer.meta::<gst_gl::GLSyncMeta>() {
                            meta.set_sync_point(&context);
                        } else {
                            let buffer = buffer.make_mut();
                            let meta = gst_gl::GLSyncMeta::add(buffer, &context);
                            meta.set_sync_point(&context);
                        }
                    }
                    event_proxy
                        .send_event(Message::Frame(info, buffer, appsink.name()))
                        .map(|()| gst::FlowSuccess::Ok)
                        .map_err(|e| {
                            element_error!(
                                appsink,
                                gst::ResourceError::Failed,
                                ("Failed to send sample to event loop: {}", e)
                            );
                            gst::FlowError::Error
                        })
                })
                .build(),
        );

        self.sink_mapping.insert(
            appsink.name(),
            AppSinkData {
                config: config,
                window_id: None,
            },
        );
    }

    pub fn create_window(
        &mut self,
        event_loop: &ActiveEventLoop,
        sink_id: &glib::GString,
    ) -> Result<WindowData> {
        let window_attributes = cfg!(windows).then(|| {
            winit::window::Window::default_attributes()
                .with_transparent(true)
                .with_title("GL rendering")
        });
        let template = glutin::config::ConfigTemplateBuilder::new().with_alpha_size(8);

        let display_builder =
            glutin_winit::DisplayBuilder::new().with_window_attributes(window_attributes);
        let (window, gl_config) = display_builder
            .build(event_loop, template, |configs| {
                configs
                    .reduce(|current, new_config| {
                        let prefer_transparency =
                            new_config.supports_transparency().unwrap_or(false)
                                & !current.supports_transparency().unwrap_or(false);
                        if prefer_transparency || new_config.num_samples() > current.num_samples() {
                            new_config
                        } else {
                            current
                        }
                    })
                    .unwrap()
            })
            .expect("Failed to build display");

        let window = window.expect("give me a window");
        let window_handle = window.window_handle().expect("a window handle");

        // XXX The display could be obtained from any object created by it, so we can query it from
        // the config.
        let gl_display = gl_config.display();
        let raw_gl_display = gl_display.raw_display();

        // The context creation part. It can be created before surface and that's how
        // it's expected in multithreaded + multiwindow operation mode, since you
        // can send NotCurrentContext, but not Surface.
        let context_attributes =
            glutin::context::ContextAttributesBuilder::new().build(Some(window_handle.as_raw()));
        // Since glutin by default tries to create OpenGL core context, which may not be
        // present we should try gles.
        let fallback_context_attributes = glutin::context::ContextAttributesBuilder::new()
            .with_context_api(glutin::context::ContextApi::Gles(None))
            .build(Some(window_handle.as_raw()));

        // There are also some old devices that support neither modern OpenGL nor GLES.
        // To support these we can try and create a 2.1 context.
        let legacy_context_attributes = glutin::context::ContextAttributesBuilder::new()
            .with_context_api(glutin::context::ContextApi::OpenGl(Some(
                glutin::context::Version::new(2, 1),
            )))
            .build(Some(window_handle.as_raw()));

        let not_current_gl_context = unsafe {
            gl_display
                .create_context(&gl_config, &context_attributes)
                .or_else(|_| {
                    gl_display
                        .create_context(&gl_config, &fallback_context_attributes)
                        .or_else(|_| {
                            gl_display.create_context(&gl_config, &legacy_context_attributes)
                        })
                })
        }
        .context("failed to create context")?;

        let raw_gl_context = not_current_gl_context.raw_context();
        println!("Using raw GL context {:?}", raw_gl_context);
        #[cfg(not(any(target_os = "linux", windows)))]
        compile_error!("This example only has Linux and Windows support");
        let api = opengl::map_gl_api(gl_config.api());
        let (raw_gl_context, gst_gl_display, platform) = match (raw_gl_display, raw_gl_context) {
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
            (glutin::display::RawDisplay::Wgl, glutin::context::RawContext::Wgl(wgl_context)) => {
                let gl_display = gst_gl::GLDisplay::new();
                (wgl_context as usize, gl_display, gst_gl::GLPlatform::WGL)
            }
            #[allow(unreachable_patterns)]
            handler => anyhow::bail!("Unsupported platform: {handler:?}."),
        };
        let glutin_context = unsafe {
            gst_gl::GLContext::new_wrapped(&gst_gl_display, raw_gl_context, platform, api)
        }
        .context("Couldn't wrap GL context")?;

        let window_id = Some(window.id());

        let window_data = WindowData {
            window: window,
            running_state: None,
            not_current_gl_context: Some(not_current_gl_context),
            glutin_context: glutin_context,
        };

        Ok(window_data)
    }

    fn configure_running_window(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_data: &mut WindowData,
    ) {
        let not_current_gl_context = window_data
            .not_current_gl_context
            .take()
            .expect("There must be a NotCurrentContext prior to Event::Resumed");

        let gl_config = not_current_gl_context.config();
        let gl_display = gl_config.display();
        // let primary_monitor = event_loop.primary_monitor();
        // for monitor in event_loop.available_monitors() {
        //     let intro = if primary_monitor.as_ref() == Some(&monitor) {
        //         "Primary monitor"
        //     } else {
        //         "Monitor"
        //     };
        //     if let Some(name) = monitor.name() {
        //         println!("{intro}: {name}");
        //     } else {
        //         println!("{intro}: [no name]");
        //     }
        //     let PhysicalSize { width, height } = monitor.size();
        //     println!(
        //         "  Current mode: {width}x{height}{}",
        //         if let Some(m_hz) = monitor.refresh_rate_millihertz() {
        //             format!(" @ {}.{} Hz", m_hz / 1000, m_hz % 1000)
        //         } else {
        //             String::new()
        //         }
        //     );
        //     println!("  Available modes (width x height x bit-depth):");
        //     for mode in monitor.video_modes() {
        //         let PhysicalSize { width, height } = mode.size();
        //         let bits = mode.bit_depth();
        //         let m_hz = mode.refresh_rate_millihertz();
        //         println!(
        //             "    {width}x{height}x{bits} @ {}.{} Hz",
        //             m_hz / 1000,
        //             m_hz % 1000
        //         );
        //     }
        // }

        //window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(event_loop.primary_monitor())));
        let attrs = window_data
            .window
            .build_surface_attributes(<_>::default())
            .unwrap();
        let gl_surface = unsafe {
            gl_config
                .display()
                .create_window_surface(&gl_config, &attrs)
                .unwrap()
        };
        // Make it current.
        let gl_context = not_current_gl_context.make_current(&gl_surface).unwrap();
        // Tell GStreamer that the context has been made current (for borrowed contexts,
        // this does not try to make it current again)
        window_data.glutin_context.activate(true).unwrap();
        window_data
            .glutin_context
            .fill_info()
            .expect("Couldn't fill context info");
        // The context needs to be current for the Renderer to set up shaders and buffers.
        // It also performs function loading, which needs a current context on WGL.
        let gl = opengl::load(&gl_display);
        // Try setting vsync.
        if let Err(res) = gl_surface.set_swap_interval(
            &gl_context,
            glutin::surface::SwapInterval::Wait(std::num::NonZeroU32::new(1).unwrap()),
        ) {
            eprintln!("Error setting vsync: {res:?}");
        }

        window_data.running_state = Some((gl, gl_context, gl_surface));
    }

    /// Should be called from within the event loop
    fn handle_message(msg: gst::Message) {
        use gst::MessageView;
        // Only handle error messages by panicking, to hard-stop the event loop
        if let MessageView::Error(err) = msg.view() {
            let src = msg
                .src()
                .map(|s| s.path_string())
                .unwrap_or_else(|| glib::GString::from("UNKNOWN"));
            let error = err.error();
            let debug = err.debug();
            panic!("Received error from {src}: {error} (debug: {debug:?})");
        }
    }
}

impl ApplicationHandler<Message> for WindowHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut windows_to_make = Vec::new();
        for (sink_id, data) in &self.sink_mapping {
            match data.window_id {
                None => {
                    let copied_id = sink_id.clone();
                    windows_to_make.push(copied_id);
                }
                _ => {}
            }
        }

        for id in windows_to_make {
            println!("Creating Window {id}");
            let mut window_data = self
                .create_window(event_loop, &id)
                .expect("we get a result");

            self.configure_running_window(event_loop, &mut window_data);

            let window_id = Some(window_data.window.id());
            self.windows.insert(window_data.window.id(), window_data);
            self.sink_mapping
                .entry(id.clone())
                .and_modify(|config| config.window_id = window_id);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                self.event_sender.send(events::RuntimeEvent::UserExit());
                self.windows.clear();
                self.sink_mapping.clear();
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // Draw.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                let window_data = self.windows.get(&id).expect("a window should exist");
                window_data.window.request_redraw();
            }
            _ => (),
        }
    }

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: Message) {
        match event {
            // Receive a frame
            Message::Frame(info, buffer, sink_id) => {
                // ! This might be slow?
                let app_data = self.sink_mapping.get(&sink_id).expect("there should be");
                let window_data = self
                    .windows
                    .get(&app_data.window_id.expect("we need a window id"))
                    .expect("a value");

                if let Ok(frame) = gst_gl::GLVideoFrame::from_buffer_readable(buffer, &info) {
                    window_data.redraw(frame);
                }
            }
            // Handle all pending messages when we are awaken by set_sync_handler
            Message::BusMessage(msg) => WindowHandler::handle_message(msg),
        }
    }
}
