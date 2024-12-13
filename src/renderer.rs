use std::ffi::{CStr, CString};
use std::ops::Deref;

use crate::gl;
use ::gl::types::GLfloat;
use glutin::prelude::GlDisplay;

pub(super) struct Shaders {
    pub(super) vertex: &'static str,
    pub(super) fragment: &'static str,
}

/// Contains name, size, stride, and offset of an attribute.
pub struct AttribInfo(String, i32, i32, usize);

/// Contains index, size, stride, and offset of an input.
pub struct ShaderInputInfo(u32, i32, i32, usize);

pub struct BufferData {
    pub vertices: Option<Vec<f32>>,
    pub attribs: Vec<AttribInfo>,
    pub inputs: Vec<ShaderInputInfo>,
}

impl BufferData {
    pub fn new() -> Self {
        Self {
            vertices: None,
            attribs: Vec::new(),
            inputs: Vec::new(),
        }
    }

    pub fn with_vertices(mut self, vertices: Vec<f32>) -> Self {
        self.vertices = Some(vertices);
        self
    }

    pub fn with_attrib(mut self, name: &str, size: i32, stride: i32, offset: usize) -> Self {
        self.attribs
            .push(AttribInfo(name.into(), size, stride, offset));
        self
    }

    pub fn with_input(mut self, index: u32, size: i32, stride: i32, offset: usize) -> Self {
        self.inputs
            .push(ShaderInputInfo(index, size, stride, offset));
        self
    }
}

pub struct Renderer {
    program: gl::types::GLuint,
    vao: gl::types::GLuint,
    vbo: Option<gl::types::GLuint>,
    gl: gl::Gl,
    draw_callback: Option<Box<dyn Fn(&Renderer)>>,
}

impl Renderer {
    pub(super) fn new<D: GlDisplay>(
        gl_display: &D,
        shaders: &Shaders,
        buffer_data: &BufferData,
        draw_callback: Option<Box<dyn Fn(&Renderer)>>,
    ) -> Self {
        unsafe {
            let gl = gl::Gl::load_with(|symbol| {
                let symbol = CString::new(symbol).unwrap();
                gl_display.get_proc_address(symbol.as_c_str()).cast()
            });

            if let Some(renderer) = get_gl_string(&gl, gl::RENDERER) {
                println!("Running on {}", renderer.to_string_lossy());
            }
            if let Some(version) = get_gl_string(&gl, gl::VERSION) {
                println!("OpenGL Version {}", version.to_string_lossy());
            }

            if let Some(shaders_version) = get_gl_string(&gl, gl::SHADING_LANGUAGE_VERSION) {
                println!("Shaders version on {}", shaders_version.to_string_lossy());
            }

            let vertex_shader = create_shader(&gl, gl::VERTEX_SHADER, shaders.vertex.as_bytes());
            let fragment_shader =
                create_shader(&gl, gl::FRAGMENT_SHADER, shaders.fragment.as_bytes());

            let program = gl.CreateProgram();

            gl.AttachShader(program, vertex_shader);
            gl.AttachShader(program, fragment_shader);

            gl.LinkProgram(program);

            gl.UseProgram(program);

            gl.DeleteShader(vertex_shader);
            gl.DeleteShader(fragment_shader);

            let mut vao = std::mem::zeroed();
            gl.GenVertexArrays(1, &mut vao);
            gl.BindVertexArray(vao);

            let mut vbo = None;
            if let Some(vertices) = &buffer_data.vertices {
                vbo = Some(std::mem::zeroed());
                gl.GenBuffers(1, vbo.as_mut().unwrap());
                gl.BindBuffer(gl::ARRAY_BUFFER, vbo.unwrap());
                gl.BufferData(
                    gl::ARRAY_BUFFER,
                    (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                    vertices.as_ptr() as *const _,
                    gl::STATIC_DRAW,
                );
            }

            for attrib in &buffer_data.attribs {
                let attrib_name = attrib.0.clone() + "\0";
                let attrib_location =
                    gl.GetAttribLocation(program, attrib_name.as_ptr() as *const _);
                gl.VertexAttribPointer(
                    attrib_location as gl::types::GLuint,
                    attrib.1 as gl::types::GLint,
                    gl::FLOAT,
                    0,
                    attrib.2 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                    (attrib.3 * std::mem::size_of::<f32>()) as *const () as *const _,
                );
                gl.EnableVertexAttribArray(attrib_location as gl::types::GLuint);
            }

            for input in &buffer_data.inputs {
                gl.VertexAttribPointer(
                    input.0 as gl::types::GLuint,
                    input.1 as gl::types::GLint,
                    gl::FLOAT,
                    0,
                    input.2 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                    (input.3 * std::mem::size_of::<f32>()) as *const () as *const _,
                );
                gl.EnableVertexAttribArray(input.0 as gl::types::GLuint);
            }

            Self {
                program,
                vao,
                vbo,
                gl,
                draw_callback,
            }
        }
    }

    pub fn uniform4f(&self, name: &str, x: f32, y: f32, z: f32, w: f32) {
        unsafe {
            let name = CString::new(name).unwrap();
            let location = self.gl.GetUniformLocation(self.program, name.as_ptr());
            self.gl.Uniform4f(location, x, y, z, w);
        }
    }

    pub fn draw(&self) {
        self.draw_with_clear_color(0.1, 0.1, 0.1, 0.9)
    }

    pub fn draw_with_clear_color(
        &self,
        red: GLfloat,
        green: GLfloat,
        blue: GLfloat,
        alpha: GLfloat,
    ) {
        unsafe {
            self.gl.UseProgram(self.program);

            if let Some(callback) = &self.draw_callback {
                callback(self);
            }

            self.gl.BindVertexArray(self.vao);
            if let Some(vbo) = self.vbo {
                self.gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
            }

            self.gl.ClearColor(red, green, blue, alpha);
            self.gl.Clear(gl::COLOR_BUFFER_BIT);
            self.gl.DrawArrays(gl::TRIANGLES, 0, 3);
        }
    }

    pub fn resize(&self, width: i32, height: i32) {
        unsafe {
            self.gl.Viewport(0, 0, width, height);
        }
    }
}

impl Deref for Renderer {
    type Target = gl::Gl;

    fn deref(&self) -> &Self::Target {
        &self.gl
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteProgram(self.program);
            if let Some(vbo) = self.vbo {
                self.gl.DeleteBuffers(1, &vbo);
            }
            self.gl.DeleteVertexArrays(1, &self.vao);
        }
    }
}

fn get_gl_string(gl: &gl::Gl, variant: gl::types::GLenum) -> Option<&'static CStr> {
    unsafe {
        let s = gl.GetString(variant);
        (!s.is_null()).then(|| CStr::from_ptr(s.cast()))
    }
}

unsafe fn create_shader(
    gl: &gl::Gl,
    shader: gl::types::GLenum,
    source: &[u8],
) -> gl::types::GLuint {
    let shader = gl.CreateShader(shader);
    gl.ShaderSource(
        shader,
        1,
        [source.as_ptr().cast()].as_ptr(),
        std::ptr::null(),
    );
    gl.CompileShader(shader);
    shader
}
