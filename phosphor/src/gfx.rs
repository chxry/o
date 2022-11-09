use std::fs::{self, File};
use std::io::BufReader;
use std::ffi::CStr;
use winit::window::{WindowBuilder, Window};
use winit::event_loop::EventLoop;
use raw_gl_context::{GlConfig, GlContext, Profile};
use glam::{Mat4, Vec3};
use obj::{Obj, TexturedVertex};
use log::info;
use crate::Result;

pub use gl;

pub struct Renderer {
  pub window: Window,
  pub context: GlContext,
}

impl Renderer {
  pub fn new(event_loop: &EventLoop<()>) -> Result<Self> {
    unsafe {
      let window = WindowBuilder::new().build(event_loop)?;
      let context = GlContext::create(
        &window,
        GlConfig {
          version: (4, 5),
          profile: Profile::Core,
          red_bits: 8,
          blue_bits: 8,
          green_bits: 8,
          alpha_bits: 0,
          depth_bits: 0,
          stencil_bits: 0,
          samples: None,
          srgb: true,
          double_buffer: true,
          vsync: true,
        },
      )
      .unwrap();
      context.make_current();
      gl::load_with(|s| context.get_proc_address(s));
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
      Ok(Self { window, context })
    }
  }

  pub fn resize(&self, size: [f32; 2]) {
    unsafe {
      gl::Viewport(0, 0, size[0] as _, size[1] as _);
      gl::Scissor(0, 0, size[0] as _, size[1] as _);
    }
  }

  pub fn clear(&self) {
    unsafe {
      gl::ClearColor(0.0, 0.0, 0.0, 1.0);
      gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
  }

  pub fn depth_test(&self, b: bool) {
    unsafe {
      (if b { gl::Enable } else { gl::Disable })(gl::DEPTH_TEST);
    }
  }
}

pub struct Shader(u32);

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

  pub fn set_mat4(&self, i: i32, val: &Mat4) {
    unsafe {
      gl::ProgramUniformMatrix4fv(self.0 as _, i, 1, gl::FALSE, val.to_cols_array().as_ptr())
    }
  }

  pub fn set_vec3(&self, i: i32, val: &Vec3) {
    unsafe { gl::ProgramUniform3fv(self.0 as _, i, 1, val.to_array().as_ptr()) }
  }
}

#[repr(C)]
pub struct Vertex {
  pos: [f32; 3],
  uv: [f32; 2],
}

pub struct Mesh {
  vert_arr: u32,
  vert_buf: u32,
  idx_buf: u32,
  len: u32,
}

impl Mesh {
  pub fn new(vertices: Vec<Vertex>, indices: Vec<u16>) -> Result<Self> {
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
      gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::TRUE, 20, 12 as _);
      Ok(Self {
        vert_arr,
        vert_buf,
        idx_buf,
        len: indices.len() as _,
      })
    }
  }

  pub fn load(path: &str) -> Result<Self> {
    let obj: Obj<TexturedVertex> = obj::load_obj(BufReader::new(File::open(path)?))?;
    Self::new(
      obj
        .vertices
        .iter()
        .map(|v| Vertex {
          pos: v.position,
          uv: [v.texture[0], v.texture[1]],
        })
        .collect(),
      obj.indices,
    )
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
  pub fn new(data: &[u8], width: u32, height: u32) -> Result<Self> {
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
        if data.is_empty() {
          std::ptr::null()
        } else {
          data.as_ptr() as _
        },
      );
      Ok(Self(tex))
    }
  }

  pub fn load(path: &str) -> Result<Self> {
    let img = image::open(path)?.to_rgba8();
    Self::new(img.as_raw(), img.width(), img.height())
  }

  pub fn bind(&self) {
    unsafe {
      gl::BindTexture(gl::TEXTURE_2D, self.0);
    }
  }

  pub fn resize(&self, width: u32, height: u32) {
    unsafe {
      gl::BindTexture(gl::TEXTURE_2D, self.0);
      gl::TexImage2D(
        gl::TEXTURE_2D,
        0,
        gl::SRGB_ALPHA as _,
        width as _,
        height as _,
        0,
        gl::RGBA,
        gl::UNSIGNED_BYTE,
        std::ptr::null(),
      );
    }
  }
}
