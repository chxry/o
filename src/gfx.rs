use std::fs::{self, File};
use std::io::BufReader;
use glutin::{ContextBuilder, WindowedContext, PossiblyCurrent};
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{EventLoop, ControlFlow};
use glutin::window::WindowBuilder;
use glam::Mat4;
use obj::{Obj, TexturedVertex};
use anyhow::Result;

pub struct Renderer {
  event_loop: EventLoop<()>,
  context: WindowedContext<PossiblyCurrent>,
  pub gl: grr::Device,
}

impl Renderer {
  pub fn new() -> Result<Self> {
    unsafe {
      let event_loop = EventLoop::new();
      let context = ContextBuilder::new()
        .build_windowed(WindowBuilder::new(), &event_loop)?
        .make_current()
        .unwrap();
      let gl = grr::Device::new(|s| context.get_proc_address(s), grr::Debug::Disable);
      gl.bind_depth_stencil_state(&grr::DepthStencil {
        depth_test: true,
        depth_write: true,
        depth_compare_op: grr::Compare::LessEqual,
        stencil_test: false,
        stencil_front: grr::StencilFace::KEEP,
        stencil_back: grr::StencilFace::KEEP,
      });
      Ok(Self {
        event_loop,
        context,
        gl,
      })
    }
  }

  pub fn run<F: Fn(&WindowedContext<PossiblyCurrent>, &grr::Device) + 'static>(
    self,
    draw: F,
  ) -> Result<()> {
    self.event_loop.run(move |event, _, control_flow| {
      self.context.window().request_redraw();
      match event {
        Event::WindowEvent { event, .. } => match event {
          WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
          WindowEvent::Resized(size) => unsafe {
            self.gl.set_viewport(
              0,
              &[grr::Viewport {
                x: 0.0,
                y: 0.0,
                w: size.width as _,
                h: size.height as _,
                n: 0.0,
                f: 1.0,
              }],
            );
            self.gl.set_scissor(
              0,
              &[grr::Region {
                x: 0,
                y: 0,
                w: size.width as _,
                h: size.height as _,
              }],
            );
          },
          _ => {}
        },
        Event::RedrawRequested(_) => {
          unsafe {
            self.gl.clear_attachment(
              grr::Framebuffer::DEFAULT,
              grr::ClearAttachment::ColorFloat(0, [0.0, 0.0, 0.0, 1.0]),
            );
            self
              .gl
              .clear_attachment(grr::Framebuffer::DEFAULT, grr::ClearAttachment::Depth(1.0));
          }
          draw(&self.context, &self.gl);
          self.context.swap_buffers().unwrap();
        }
        _ => {}
      }
    })
  }
}

pub struct Shader(grr::Pipeline);

impl Shader {
  pub fn new(gl: &grr::Device, vert_path: &str, frag_path: &str) -> Result<Self> {
    unsafe {
      let vert = gl.create_shader(
        grr::ShaderStage::Vertex,
        fs::read_to_string(vert_path)?.as_bytes(),
        grr::ShaderFlags::VERBOSE,
      )?;
      let frag = gl.create_shader(
        grr::ShaderStage::Fragment,
        fs::read_to_string(frag_path)?.as_bytes(),
        grr::ShaderFlags::VERBOSE,
      )?;
      Ok(Self(gl.create_graphics_pipeline(
        grr::VertexPipelineDesc {
          vertex_shader: vert,
          tessellation_control_shader: None,
          tessellation_evaluation_shader: None,
          geometry_shader: None,
          fragment_shader: Some(frag),
        },
        grr::PipelineFlags::VERBOSE,
      )?))
    }
  }

  pub fn bind(&self, gl: &grr::Device) {
    unsafe {
      gl.bind_pipeline(self.0);
    }
  }

  pub fn set_mat4(&self, gl: &grr::Device, i: u32, val: Mat4) {
    unsafe {
      gl.bind_uniform_constants(self.0, i, &[grr::Constant::Mat4x4(val.to_cols_array_2d())]);
    }
  }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
  pos: [f32; 3],
  uv: [f32; 2],
}

impl Vertex {
  const STRIDE: u32 = 20;
  const ATTR: [grr::VertexAttributeDesc; 2] = [
    grr::VertexAttributeDesc {
      location: 0,
      binding: 0,
      format: grr::VertexFormat::Xyz32Float,
      offset: 0,
    },
    grr::VertexAttributeDesc {
      location: 1,
      binding: 0,
      format: grr::VertexFormat::Xy32Float,
      offset: 12,
    },
  ];
}

pub struct Mesh {
  arr: grr::VertexArray,
  vert_buf: grr::Buffer,
  idx_buf: grr::Buffer,
  len: u32,
}

impl Mesh {
  pub fn new(gl: &grr::Device, vertices: Vec<Vertex>, indices: Vec<u16>) -> Result<Self> {
    unsafe {
      Ok(Self {
        arr: gl.create_vertex_array(&Vertex::ATTR)?,
        vert_buf: gl
          .create_buffer_from_host(bytemuck::cast_slice(&vertices), grr::MemoryFlags::empty())?,
        idx_buf: gl
          .create_buffer_from_host(bytemuck::cast_slice(&indices), grr::MemoryFlags::empty())?,
        len: indices.len() as u32,
      })
    }
  }

  pub fn load(gl: &grr::Device, path: &str) -> Result<Self> {
    let obj: Obj<TexturedVertex> = obj::load_obj(BufReader::new(File::open(path)?))?;
    Self::new(
      gl,
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

  pub fn draw(&self, gl: &grr::Device) {
    unsafe {
      gl.bind_vertex_array(self.arr);
      gl.bind_vertex_buffers(
        self.arr,
        0,
        &[grr::VertexBufferView {
          buffer: self.vert_buf,
          offset: 0,
          stride: Vertex::STRIDE,
          input_rate: grr::InputRate::Vertex,
        }],
      );
      gl.bind_index_buffer(self.arr, self.idx_buf);
      gl.draw_indexed(
        grr::Primitive::Triangles,
        grr::IndexTy::U16,
        0..self.len,
        0..1,
        0,
      );
    }
  }
}

pub struct Texture(grr::ImageView);

impl Texture {
  pub fn new(gl: &grr::Device, path: &str) -> Result<Self> {
    unsafe {
      let img = image::open(path)?.to_rgba8();
      let (tex, view) = gl.create_image_and_view(
        grr::ImageType::D2 {
          width: img.width(),
          height: img.height(),
          layers: 1,
          samples: 1,
        },
        grr::Format::R8G8B8A8_SRGB,
        1,
      )?;
      gl.copy_host_to_image(
        img.as_raw(),
        tex,
        grr::HostImageCopy {
          host_layout: grr::MemoryLayout {
            base_format: grr::BaseFormat::RGBA,
            format_layout: grr::FormatLayout::U8,
            row_length: img.width(),
            image_height: img.height(),
            alignment: 4,
          },
          image_subresource: grr::SubresourceLayers {
            level: 0,
            layers: 0..1,
          },
          image_offset: grr::Offset { x: 0, y: 0, z: 0 },
          image_extent: grr::Extent {
            width: img.width(),
            height: img.height(),
            depth: 1,
          },
        },
      );
      Ok(Self(view))
    }
  }

  pub fn bind(&self, gl: &grr::Device) {
    unsafe {
      gl.bind_image_views(0, &[self.0]);
    }
  }
}
