//! This example demonstrates how to output GL textures, within an EGL/X11 context provided by the
//! application, and render those textures in the GL application.
//!
//! This example follow common patterns from `glutin`:
//! <https://github.com/rust-windowing/glutin/blob/master/glutin_examples/src/lib.rs>
// {videotestsrc} - { glsinkbin }
use std::{
    ffi::{CStr, CString},
    mem, ptr,
};
#[rustfmt::skip]
static VERTICES: [f32; 20] = [
     1.0,  1.0, 0.0, 1.0, 0.0,
    -1.0,  1.0, 0.0, 0.0, 0.0,
    -1.0, -1.0, 0.0, 0.0, 1.0,
     1.0, -1.0, 0.0, 1.0, 1.0,
];
static INDICES: [u16; 6] = [0, 1, 2, 0, 2, 3];
#[rustfmt::skip]
static IDENTITY: [f32; 16] = [
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0,
];
const VS_SRC: &[u8] = c"
uniform mat4 u_transformation;
attribute vec4 a_position;
attribute vec2 a_texcoord;
varying vec2 v_texcoord;
void main() {
    gl_Position = u_transformation * a_position;
    v_texcoord = a_texcoord;
}"
.to_bytes();
const FS_SRC: &[u8] = c"
#ifdef GL_ES
precision mediump float;
#endif
varying vec2 v_texcoord;
uniform sampler2D tex;
void main() {
    gl_FragColor = texture2D(tex, v_texcoord);
}"
.to_bytes();
#[allow(clippy::unreadable_literal)]
#[allow(clippy::unused_unit)]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::manual_non_exhaustive)]
#[allow(clippy::upper_case_acronyms)]
#[allow(clippy::missing_transmute_annotations)]
pub(crate) mod gl {
    pub use self::Gles2 as Gl;
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}
pub struct Gl {
    gl: gl::Gl,
    program: gl::types::GLuint,
    attr_position: gl::types::GLint,
    attr_texture: gl::types::GLint,
    vao: Option<gl::types::GLuint>,
    vertex_buffer: gl::types::GLuint,
    vbo_indices: gl::types::GLuint,
}
impl Gl {
    pub fn draw_frame(&self, texture_id: gl::types::GLuint) {
        unsafe {
            // render
            self.gl.ClearColor(0.0, 0.0, 0.0, 1.0);
            self.gl.Clear(gl::COLOR_BUFFER_BIT);
            self.gl.BlendColor(0.0, 0.0, 0.0, 1.0);
            if self.gl.BlendFuncSeparate.is_loaded() {
                self.gl.BlendFuncSeparate(
                    gl::SRC_ALPHA,
                    gl::CONSTANT_COLOR,
                    gl::ONE,
                    gl::ONE_MINUS_SRC_ALPHA,
                );
            } else {
                self.gl.BlendFunc(gl::SRC_ALPHA, gl::CONSTANT_COLOR);
            }
            self.gl.BlendEquation(gl::FUNC_ADD);
            self.gl.Enable(gl::BLEND);
            self.gl.UseProgram(self.program);
            if self.gl.BindVertexArray.is_loaded() {
                self.gl.BindVertexArray(self.vao.unwrap());
            }
            {
                self.gl
                    .BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.vbo_indices);
                self.gl.BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer);
                // Load the vertex position
                self.gl.VertexAttribPointer(
                    self.attr_position as gl::types::GLuint,
                    3,
                    gl::FLOAT,
                    gl::FALSE,
                    (5 * mem::size_of::<f32>()) as gl::types::GLsizei,
                    ptr::null(),
                );
                // Load the texture coordinate
                self.gl.VertexAttribPointer(
                    self.attr_texture as gl::types::GLuint,
                    2,
                    gl::FLOAT,
                    gl::FALSE,
                    (5 * mem::size_of::<f32>()) as gl::types::GLsizei,
                    (3 * mem::size_of::<f32>()) as *const () as *const _,
                );
                self.gl.EnableVertexAttribArray(self.attr_position as _);
                self.gl.EnableVertexAttribArray(self.attr_texture as _);
            }
            self.gl.ActiveTexture(gl::TEXTURE0);
            self.gl.BindTexture(gl::TEXTURE_2D, texture_id);
            let location = self
                .gl
                .GetUniformLocation(self.program, c"tex".as_ptr() as *const _);
            self.gl.Uniform1i(location, 0);
            let location = self
                .gl
                .GetUniformLocation(self.program, c"u_transformation".as_ptr() as *const _);
            self.gl
                .UniformMatrix4fv(location, 1, gl::FALSE, IDENTITY.as_ptr() as *const _);
            self.gl
                .DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_SHORT, ptr::null());
            self.gl.BindTexture(gl::TEXTURE_2D, 0);
            self.gl.UseProgram(0);
            if self.gl.BindVertexArray.is_loaded() {
                self.gl.BindVertexArray(0);
            }
            {
                self.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
                self.gl.BindBuffer(gl::ARRAY_BUFFER, 0);
                self.gl.DisableVertexAttribArray(self.attr_position as _);
                self.gl.DisableVertexAttribArray(self.attr_texture as _);
            }
        }
    }
    pub fn resize(&self, size: winit::dpi::PhysicalSize<u32>) {
        unsafe {
            self.gl
                .Viewport(0, 0, size.width as i32, size.height as i32);
        }
    }
}
pub fn load(gl_display: &impl glutin::display::GlDisplay) -> Gl {
    let gl = gl::Gl::load_with(|symbol| {
        let symbol = CString::new(symbol).unwrap();
        gl_display.get_proc_address(&symbol).cast()
    });
    let version = unsafe {
        let version = gl.GetString(gl::VERSION);
        assert!(!version.is_null());
        let version = CStr::from_ptr(version.cast());
        version.to_string_lossy()
    };
    println!("OpenGL version {version}");
    let (program, attr_position, attr_texture, vao, vertex_buffer, vbo_indices) = unsafe {
        let vs = gl.CreateShader(gl::VERTEX_SHADER);
        gl.ShaderSource(vs, 1, [VS_SRC.as_ptr() as *const _].as_ptr(), ptr::null());
        gl.CompileShader(vs);
        let fs = gl.CreateShader(gl::FRAGMENT_SHADER);
        gl.ShaderSource(fs, 1, [FS_SRC.as_ptr() as *const _].as_ptr(), ptr::null());
        gl.CompileShader(fs);
        let program = gl.CreateProgram();
        gl.AttachShader(program, vs);
        gl.AttachShader(program, fs);
        gl.LinkProgram(program);
        {
            let mut success = 1;
            gl.GetProgramiv(program, gl::LINK_STATUS, &mut success);
            assert_ne!(success, 0);
            assert_eq!(gl.GetError(), 0);
        }
        let attr_position = gl.GetAttribLocation(program, c"a_position".as_ptr() as *const _);
        let attr_texture = gl.GetAttribLocation(program, c"a_texcoord".as_ptr() as *const _);
        let vao = if gl.BindVertexArray.is_loaded() {
            let mut vao = mem::MaybeUninit::uninit();
            gl.GenVertexArrays(1, vao.as_mut_ptr());
            let vao = vao.assume_init();
            gl.BindVertexArray(vao);
            Some(vao)
        } else {
            None
        };
        let mut vertex_buffer = mem::MaybeUninit::uninit();
        gl.GenBuffers(1, vertex_buffer.as_mut_ptr());
        let vertex_buffer = vertex_buffer.assume_init();
        gl.BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);
        gl.BufferData(
            gl::ARRAY_BUFFER,
            (VERTICES.len() * mem::size_of::<f32>()) as gl::types::GLsizeiptr,
            VERTICES.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );
        let mut vbo_indices = mem::MaybeUninit::uninit();
        gl.GenBuffers(1, vbo_indices.as_mut_ptr());
        let vbo_indices = vbo_indices.assume_init();
        gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, vbo_indices);
        gl.BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (INDICES.len() * mem::size_of::<u16>()) as gl::types::GLsizeiptr,
            INDICES.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );
        if gl.BindVertexArray.is_loaded() {
            gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, vbo_indices);
            gl.BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);
            // Load the vertex position
            gl.VertexAttribPointer(
                attr_position as gl::types::GLuint,
                3,
                gl::FLOAT,
                gl::FALSE,
                (5 * mem::size_of::<f32>()) as gl::types::GLsizei,
                ptr::null(),
            );
            // Load the texture coordinate
            gl.VertexAttribPointer(
                attr_texture as gl::types::GLuint,
                2,
                gl::FLOAT,
                gl::FALSE,
                (5 * mem::size_of::<f32>()) as gl::types::GLsizei,
                (3 * mem::size_of::<f32>()) as *const () as *const _,
            );
            gl.EnableVertexAttribArray(attr_position as _);
            gl.EnableVertexAttribArray(attr_texture as _);
            gl.BindVertexArray(0);
        }
        gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        gl.BindBuffer(gl::ARRAY_BUFFER, 0);
        assert_eq!(gl.GetError(), 0);
        (
            program,
            attr_position,
            attr_texture,
            vao,
            vertex_buffer,
            vbo_indices,
        )
    };
    Gl {
        gl,
        program,
        attr_position,
        attr_texture,
        vao,
        vertex_buffer,
        vbo_indices,
    }
}

/// Converts from <https://docs.rs/glutin/latest/glutin/config/struct.Api.html> to
/// <https://gstreamer.freedesktop.org/documentation/gl/gstglapi.html?gi-language=c#GstGLAPI>.
pub fn map_gl_api(api: glutin::config::Api) -> gst_gl::GLAPI {
    use glutin::config::Api;
    use gst_gl::GLAPI;
    let mut gst_gl_api = GLAPI::empty();
    // In gstreamer:
    // GLAPI::OPENGL: Desktop OpenGL up to and including 3.1. The compatibility profile when the OpenGL version is >= 3.2
    // GLAPI::OPENGL3: Desktop OpenGL >= 3.2 core profile
    // In glutin, API::OPENGL is set for every context API, except EGL where it is set based on
    // EGL_RENDERABLE_TYPE containing EGL_OPENGL_BIT:
    // https://registry.khronos.org/EGL/sdk/docs/man/html/eglChooseConfig.xhtml
    gst_gl_api.set(GLAPI::OPENGL | GLAPI::OPENGL3, api.contains(Api::OPENGL));
    gst_gl_api.set(GLAPI::GLES1, api.contains(Api::GLES1));
    // OpenGL ES 2.x and 3.x
    gst_gl_api.set(GLAPI::GLES2, api.intersects(Api::GLES2 | Api::GLES3));
    gst_gl_api
}
