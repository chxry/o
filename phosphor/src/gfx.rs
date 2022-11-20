use std::fs::{self, File};
use std::io::BufReader;
use std::ffi::{CStr, CString};
use std::sync::mpsc::Receiver;
use glfw::{Context, WindowEvent};
use glam::{Mat4, Vec3};
use obj::{Obj, TexturedVertex};
use log::info;
use crate::Result;

pub use gl;

pub struct Renderer {
  pub glfw: glfw::Glfw,
  pub window: glfw::Window,
  pub events: Receiver<(f64, WindowEvent)>,
}

impl Renderer {
  pub fn new() -> Result<Self> {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
      glfw::OpenGlProfileHint::Core,
    ));
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
    let (mut window, events) = glfw
      .create_window(1400, 800, "phosphor", glfw::WindowMode::Windowed)
      .unwrap();
    window.make_current();
    window.set_all_polling(true);
    gl::load_with(|s| window.get_proc_address(s));
    unsafe {
      gl::Enable(gl::FRAMEBUFFER_SRGB);
      gl::Enable(gl::DEPTH_TEST);
      gl::Enable(gl::SCISSOR_TEST);
      gl::Enable(gl::BLEND);
      gl::BlendFuncSeparate(
        gl::SRC_ALPHA,
        gl::ONE_MINUS_SRC_ALPHA,
        gl::ONE,
        gl::ONE_MINUS_SRC_ALPHA,
      );
      let version = CStr::from_ptr(gl::GetString(gl::VERSION) as _).to_str()?;
      let renderer = CStr::from_ptr(gl::GetString(gl::RENDERER) as _).to_str()?;
      info!("Created OpenGL {} renderer on {}.", version, renderer);
    }
    Ok(Self {
      glfw,
      window,
      events,
    })
  }

  pub fn resize(&self, w: i32, h: i32) {
    unsafe {
      gl::Viewport(0, 0, w, h);
      gl::Scissor(0, 0, w, h);
    }
  }

  pub fn clear(&self) {
    unsafe {
      gl::ClearColor(0.0, 0.0, 0.0, 1.0);
      gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
  }
}

pub struct Shader(pub u32);

impl Shader {
  pub fn new(vert_path: &str, frag_path: &str) -> Result<Self> {
    unsafe {
      let vert = gl::CreateShader(gl::VERTEX_SHADER);
      let vert_src = fs::read_to_string(vert_path)?;
      gl::ShaderSource(
        vert,
        1,
        &(vert_src.as_bytes().as_ptr().cast()),
        &(vert_src.len().try_into().unwrap()),
      );
      gl::CompileShader(vert);

      let frag_src = fs::read_to_string(frag_path)?;
      let frag = gl::CreateShader(gl::FRAGMENT_SHADER);
      gl::ShaderSource(
        frag,
        1,
        &(frag_src.as_bytes().as_ptr().cast()),
        &(frag_src.len().try_into().unwrap()),
      );
      gl::CompileShader(frag);

      let program = gl::CreateProgram();
      gl::AttachShader(program, vert);
      gl::AttachShader(program, frag);
      gl::LinkProgram(program);
      gl::DeleteShader(vert);
      gl::DeleteShader(frag);
      Ok(Self(program))
    }
  }

  pub fn bind(&self) {
    unsafe { gl::UseProgram(self.0) }
  }

  fn get_loc(&self, name: &'static str) -> i32 {
    let c = CString::new(name).unwrap();
    unsafe { gl::GetUniformLocation(self.0, c.as_ptr() as _) }
  }

  pub fn set_mat4(&self, name: &'static str, val: &Mat4) {
    unsafe {
      gl::ProgramUniformMatrix4fv(
        self.0 as _,
        self.get_loc(name),
        1,
        gl::FALSE,
        val.to_cols_array().as_ptr(),
      )
    }
  }

  pub fn set_vec3(&self, name: &'static str, val: &Vec3) {
    unsafe { gl::ProgramUniform3fv(self.0 as _, self.get_loc(name), 1, val.to_array().as_ptr()) }
  }
}

#[repr(C)]
pub struct Vertex {
  pub pos: [f32; 3],
  pub uv: [f32; 2],
}

pub struct Mesh {
  pub vert_arr: u32,
  pub vert_buf: u32,
  pub idx_buf: u32,
  pub len: u32,
}

impl Mesh {
  pub fn new(vertices: &[Vertex], indices: &[u16]) -> Self {
    unsafe {
      let mut vert_arr = 0;
      gl::GenVertexArrays(1, &mut vert_arr);
      gl::BindVertexArray(vert_arr);
      let mut vert_buf = 0;
      gl::GenBuffers(1, &mut vert_buf);
      gl::BindBuffer(gl::ARRAY_BUFFER, vert_buf);
      gl::BufferData(
        gl::ARRAY_BUFFER,
        (vertices.len() * 20) as _,
        vertices.as_ptr() as _,
        gl::STATIC_DRAW,
      );
      let mut idx_buf = 0;
      gl::GenBuffers(1, &mut idx_buf);
      gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, idx_buf);
      gl::BufferData(
        gl::ELEMENT_ARRAY_BUFFER,
        (indices.len() * 2) as _,
        indices.as_ptr() as _,
        gl::STATIC_DRAW,
      );
      gl::EnableVertexAttribArray(0);
      gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 20, 0 as _);
      gl::EnableVertexAttribArray(1);
      gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 20, 12 as _);
      Self {
        vert_arr,
        vert_buf,
        idx_buf,
        len: indices.len() as _,
      }
    }
  }

  pub fn load(path: &str) -> Result<Self> {
    let obj: Obj<TexturedVertex> = obj::load_obj(BufReader::new(File::open(path)?))?;
    Ok(Self::new(
      &obj
        .vertices
        .iter()
        .map(|v| Vertex {
          pos: v.position,
          uv: [v.texture[0], v.texture[1]],
        })
        .collect::<Vec<_>>(),
      &obj.indices,
    ))
  }

  pub fn draw(&self) {
    unsafe {
      gl::BindVertexArray(self.vert_arr);
      gl::DrawElements(
        gl::TRIANGLES,
        self.len as _,
        gl::UNSIGNED_SHORT,
        std::ptr::null(),
      );
    }
  }
}

pub struct Texture(pub u32);

impl Texture {
  pub fn new(data: &[u8], width: u32, height: u32) -> Self {
    unsafe {
      let mut tex = 0;
      gl::GenTextures(1, &mut tex);
      gl::BindTexture(gl::TEXTURE_2D, tex);
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);
      gl::TexImage2D(
        gl::TEXTURE_2D,
        0,
        gl::SRGB_ALPHA as _,
        width as _,
        height as _,
        0,
        gl::RGBA,
        gl::UNSIGNED_BYTE,
        data.as_ptr() as _,
      );
      Self(tex)
    }
  }

  pub fn empty() -> Self {
    unsafe {
      let mut tex = 0;
      gl::GenTextures(1, &mut tex);
      gl::BindTexture(gl::TEXTURE_2D, tex);
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);
      gl::TexImage2D(
        gl::TEXTURE_2D,
        0,
        gl::SRGB_ALPHA as _,
        0,
        0,
        0,
        gl::RGBA,
        gl::UNSIGNED_BYTE,
        0 as _,
      );
      Self(tex)
    }
  }

  pub fn load(path: &str) -> Result<Self> {
    let img = image::open(path)?.to_rgba8();
    Ok(Self::new(img.as_raw(), img.width(), img.height()))
  }

  pub fn bind(&self) {
    unsafe {
      gl::BindTexture(gl::TEXTURE_2D, self.0);
    }
  }

  pub fn resize(&self, width: u32, height: u32) {
    unsafe {
      self.bind();
      gl::TexImage2D(
        gl::TEXTURE_2D,
        0,
        gl::SRGB_ALPHA as _,
        width as _,
        height as _,
        0,
        gl::RGBA,
        gl::UNSIGNED_BYTE,
        0 as _,
      );
    }
  }
}

#[derive(Copy, Clone)]
pub struct Framebuffer {
  pub fb: u32,
  pub rb: u32,
}

impl Framebuffer {
  pub const DEFAULT: Framebuffer = Self { fb: 0, rb: 0 };

  pub fn new() -> Self {
    unsafe {
      let mut fb = 0;
      gl::GenFramebuffers(1, &mut fb);
      gl::BindFramebuffer(gl::FRAMEBUFFER, fb);
      let mut rb = 0;
      gl::GenRenderbuffers(1, &mut rb);
      gl::BindRenderbuffer(gl::RENDERBUFFER, rb);
      gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, 0, 0);
      gl::FramebufferRenderbuffer(
        gl::FRAMEBUFFER,
        gl::DEPTH_STENCIL_ATTACHMENT,
        gl::RENDERBUFFER,
        rb,
      );
      Self { fb, rb }
    }
  }

  pub fn bind(&self) {
    unsafe {
      gl::BindFramebuffer(gl::FRAMEBUFFER, self.fb);
    }
  }

  pub fn bind_tex(&self, tex: &Texture) {
    unsafe {
      self.bind();
      gl::FramebufferTexture2D(
        gl::FRAMEBUFFER,
        gl::COLOR_ATTACHMENT0,
        gl::TEXTURE_2D,
        tex.0,
        0,
      );
    }
  }

  pub fn resize(&self, width: i32, height: i32) {
    unsafe {
      gl::BindRenderbuffer(gl::RENDERBUFFER, self.rb);
      gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, width, height);
    }
  }
}
