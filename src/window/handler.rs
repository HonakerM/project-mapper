use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::mpsc;

use crate::config::events;
use crate::config::sink::Resolution;
use crate::opengl::{self, gl};
use anyhow::{Context, Error, Result};
use glutin::config::{GetGlConfig, GlConfig};
use glutin::context::AsRawContext;
use glutin::display::{AsRawDisplay, GetGlDisplay};
use glutin::prelude::{GlDisplay, NotCurrentGlContext, PossiblyCurrentGlContext};
use glutin::surface::GlSurface;
use glutin_winit::GlWindow;
use gst::prelude::{ElementExt, GstObjectExt, PadExt, PadExtManual};
use gst::{PadProbeReturn, PadProbeType, QueryViewMut, element_error};
use gst_gl::GLVideoFrameExt;
use gst_gl::prelude::GLContextExt;
use gst_video::VideoFrameExt;
use raw_window_handle::HasWindowHandle;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::monitor::{MonitorHandle, VideoModeHandle};
use winit::window::{Window, WindowId};

struct MonitorData {
    name: String,
    monitor: MonitorHandle,
    mode_lookup: HashMap<Resolution, HashMap<u32, VideoModeHandle>>,
}

struct WindowData {
    window: Window,
    running_state: Option<(
        opengl::Gl,
        glutin::context::PossiblyCurrentContext,
        glutin::surface::Surface<glutin::surface::WindowSurface>,
    )>,
    not_current_gl_context: Option<glutin::context::NotCurrentContext>,
    glutin_context: gst_gl::GLContext,
    config: crate::config::sink::SinkType,
}

impl WindowData {
    /// Should be called from within the event loop
    pub fn redraw(&self, current_frame: gst_gl::GLVideoFrame<gst_gl::gl_video_frame::Readable>) {
        if let Some((gl, gl_context, gl_surface)) = &self.running_state {
            gl_context
                .make_current(gl_surface)
                .expect("could not make current");

            let sync_meta = current_frame.buffer().meta::<gst_gl::GLSyncMeta>().unwrap();
            sync_meta.wait(&self.glutin_context);
            if let Ok(texture) = current_frame.texture_id(0) {
                gl.draw_frame(texture as gl::types::GLuint);
            }
            gl_surface.swap_buffers(gl_context).unwrap();
        }
    }

    pub fn resize(&self, size: PhysicalSize<u32>) {
        if let Some((gl, gl_context, gl_surface)) = &self.running_state {
            gl_context
                .make_current(gl_surface)
                .expect("could not make current");

            gl_surface.resize(
                gl_context,
                // XXX Ignore minimizing
                NonZeroU32::new(size.width).unwrap(),
                NonZeroU32::new(size.height).unwrap(),
            );
            gl.resize(size);
        }
    }
}

#[derive(Debug)]
pub(crate) enum Message {
    Frame(gst_video::VideoInfo, gst::Buffer, WindowId),
    BusMessage(gst::Message),
}

pub struct WindowHandler {
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
            windows: HashMap::new(),
            event_proxy: event_proxy,
            event_sender: event_sender,
        }
    }

    pub fn add_sink(
        &mut self,
        sink_name: glib::GString,
        appsink: gst_app::AppSink,
        event_loop: &winit::event_loop::EventLoop<Message>,
        sink_info: crate::config::sink::SinkType,
    ) {
        let event_proxy = self.event_proxy.clone();
        let appsink_id = sink_name.clone();

        let window_data = self
            .create_window(appsink_id.clone(), &appsink, event_loop, sink_info)
            .expect("we get a result");
        let window_id = window_data.window.id();

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
                        .send_event(Message::Frame(info, buffer, window_id))
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

        self.windows.insert(window_id.clone(), window_data);
    }

    fn create_window(
        &mut self,
        name: glib::GString,
        appsink: &gst_app::AppSink,
        event_loop: &winit::event_loop::EventLoop<Message>,
        sink_info: crate::config::sink::SinkType,
    ) -> Result<WindowData> {
        let mut window_attributes = winit::window::Window::default_attributes()
            .with_transparent(true)
            .with_title(name.clone().to_string());

        let template = glutin::config::ConfigTemplateBuilder::new().with_alpha_size(8);

        let display_builder =
            glutin_winit::DisplayBuilder::new().with_window_attributes(Some(window_attributes));
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
        println!(
            "Picked a config with {} samples and transparency {}. Pixel format: {:?}",
            gl_config.num_samples(),
            gl_config.supports_transparency().unwrap_or(false),
            gl_config.color_buffer_type()
        );
        println!("Config supports GL API(s) {:?}", gl_config.api());

        let window = window.expect("give me a window");
        let window_handle = window.window_handle().expect("a window handle");

        // XXX The display could be obtained from any object created by it, so we can query it from
        // the config.
        let gl_display = gl_config.display();
        let raw_gl_display = gl_display.raw_display();

        println!("Using raw display connection {:?}", raw_gl_display);

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

        {
            // Make a new context that isn't the wrapped glutin context so that it can be made
            // current on a new "gstglcontext" thread (see `gst_gl_context_create_thread()`), while
            // the wrapped glutin context is made current on the winit event loop thread (this main
            // thread).
            let shared_context = gst_gl::GLContext::new(&gst_gl_display);
            shared_context
                .create(Some(&glutin_context))
                .context("Couldn't share wrapped Glutin GL context with new GL context")?;
            // Return the shared `GLContext` out of a pad probe for "gst.gl.local_context" to
            // make the underlying pipeline use it directly, instead of creating a new GL context
            // that is *shared* with the resulting context from a context `Query` (among other
            // elements) or `NeedContext` bus message for "gst.gl.app_context", as documented for
            // `gst_gl_ensure_element_data()`.
            //
            // On Windows, such context sharing calls `wglShareLists()` which fails on certain
            // drivers when one of the contexts is already current on another thread.  This would
            // happen because the pipeline and specifically the aforementioned "gstglcontext"
            // thread would be initialized asynchronously from the winit loop which makes our glutin
            // context current.  By calling `GLContext::create()` above, context sharing happens
            // directly.
            //
            // An alternative approach would be using `gst_gl::GLDisplay::add_context()` to store
            // the context inside `GLDisplay`, but the pad probe takes precedence.
            // While the pad probe could be installed anywhere, it makes logical sense to insert it
            // on the appsink where the images are extracted and displayed to a window via the same
            // GL contexts.
            appsink
                .static_pad("sink")
                .unwrap()
                .add_probe(PadProbeType::QUERY_DOWNSTREAM, move |pad, probe_info| {
                    if let Some(q) = probe_info.query_mut() {
                        if let QueryViewMut::Context(cq) = q.view_mut() {
                            if gst_gl::functions::gl_handle_context_query(
                                &pad.parent_element().unwrap(),
                                cq,
                                Some(&gst_gl_display),
                                Some(&shared_context),
                                None::<&gst_gl::GLContext>,
                            ) {
                                return PadProbeReturn::Handled;
                            }
                        }
                    }
                    PadProbeReturn::Ok
                })
                .unwrap();
        }

        let window_data = WindowData {
            window: window,
            running_state: None,
            not_current_gl_context: Some(not_current_gl_context),
            glutin_context: glutin_context,
            config: sink_info,
        };

        Ok(window_data)
    }

    fn configure_running_window(
        window_data: &mut WindowData,
        monitor_data: &HashMap<String, MonitorData>,
    ) {
        let not_current_gl_context = window_data
            .not_current_gl_context
            .take()
            .expect("There must be a NotCurrentContext prior to Event::Resumed");

        let gl_config = not_current_gl_context.config();
        let gl_display = gl_config.display();

        //window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(event_loop.primary_monitor())));
        WindowHandler::configure_fullscreen(window_data, monitor_data)
            .expect("you did bad cause error");

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

    fn configure_fullscreen(
        window_data: &mut WindowData,
        monitor_data: &HashMap<String, MonitorData>,
    ) -> Result<()> {
        match &window_data.config {
            crate::config::sink::SinkType::OpenGLWindow { full_screen } => match full_screen {
                crate::config::sink::FullScreenMode::Borderless { name } => {
                    match &monitor_data.get(name) {
                        None => {
                            let monitor_names = monitor_data.keys();
                            Err(Error::msg(format!(
                                "Unkown monitor name {name} supported monitors: {monitor_names:?}"
                            )))
                        }
                        Some(monitor) => {
                            window_data.window.set_fullscreen(Some(
                                winit::window::Fullscreen::Borderless(Some(
                                    monitor.monitor.clone(),
                                )),
                            ));
                            Ok(())
                        }
                    }
                }
                crate::config::sink::FullScreenMode::Exclusive { info } => {
                    match &monitor_data.get(&info.name) {
                        None => {
                            let attempted_name = &info.name;
                            let monitor_names = monitor_data.keys();
                            Err(Error::msg(format!(
                                "Unkown monitor name {attempted_name} supported monitors: {monitor_names:?}"
                            )))
                        }
                        Some(monitor) => {
                            let video_mode = monitor
                                .mode_lookup
                                .get(&info.resolution)
                                .ok_or(Error::msg("unknown resolution"))?
                                .get(&info.refresh_rate)
                                .ok_or(Error::msg("unknown refresh rate"))?;
                            window_data.window.set_fullscreen(Some(
                                winit::window::Fullscreen::Exclusive(video_mode.clone()),
                            ));
                            Ok(())
                        }
                    }
                }
                crate::config::sink::FullScreenMode::Windowed {} => Ok(()),
            },
        }
    }

    fn gather_monitor_info(event_loop: &ActiveEventLoop) -> HashMap<String, MonitorData> {
        let mut monitor_map = HashMap::new();
        for monitor in event_loop.available_monitors() {
            let mut resolution_map: HashMap<Resolution, HashMap<u32, VideoModeHandle>> =
                HashMap::new();
            for monitor_handle in monitor.video_modes() {
                let resolution = Resolution::from_size(monitor.size());

                if !resolution_map.contains_key(&resolution) {
                    resolution_map.insert(resolution.clone(), HashMap::new());
                }

                let frequency_map = resolution_map.get_mut(&resolution).expect("we just added");

                let refresh_rate = monitor_handle.refresh_rate_millihertz();

                frequency_map.insert(refresh_rate, monitor_handle);
            }
            let monitor_name = monitor.name().expect("we have a name");
            let monitor_data = MonitorData {
                name: monitor_name.clone(),
                monitor: monitor,
                mode_lookup: resolution_map,
            };
            monitor_map.insert(monitor_name, monitor_data);
        }
        monitor_map
    }
}

impl ApplicationHandler<Message> for WindowHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let monitor_data = WindowHandler::gather_monitor_info(event_loop);
        for (_, windows) in self.windows.iter_mut() {
            WindowHandler::configure_running_window(windows, &monitor_data);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                self.event_sender.send(events::RuntimeEvent::UserExit());
                self.windows.clear();
                event_loop.exit();
            }
            winit::event::WindowEvent::Resized(size) => {
                // Some platforms like EGL require resizing GL surface to update the size
                // Notable platforms here are Wayland and macOS, other don't require it
                // and the function is no-op, but it's wise to resize it for portability
                // reasons.
                let window_data = self.windows.get(&id).expect("a window should exist");
                window_data.resize(size);
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
            Message::Frame(info, buffer, window_id) => {
                // ! This might be slow?
                let window_data = self.windows.get(&window_id).expect("a value");

                if let Ok(frame) = gst_gl::GLVideoFrame::from_buffer_readable(buffer, &info) {
                    window_data.redraw(frame);
                }
            }
            // Handle all pending messages when we are awaken by set_sync_handler
            Message::BusMessage(msg) => WindowHandler::handle_message(msg),
        }
    }
}
