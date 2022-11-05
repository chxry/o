use std::fs::{self, File};
use std::io::BufReader;
use std::ffi::CStr;
use glutin::{ContextBuilder, WindowedContext, PossiblyCurrent};
use glutin::window::WindowBuilder;
use glutin::dpi::PhysicalSize;
use glutin::event_loop::EventLoop;
use glam::{Mat4, Vec3};
use obj::{Obj, TexturedVertex};
use log::info;
use crate::Result;

pub struct Renderer {
  pub context: WindowedContext<PossiblyCurrent>,
  pub gl: grr::Device,
}

impl Renderer {
  pub fn new(event_loop: &EventLoop<()>) -> Result<Self> {
    unsafe {
      let context = ContextBuilder::new()
        .with_vsync(true)
        .build_windowed(WindowBuilder::new(), event_loop)?
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
      gl.bind_color_blend_state(&grr::ColorBlend {
        attachments: vec![grr::ColorBlendAttachment {
          blend_enable: true,
          color: grr::BlendChannel {
            src_factor: grr::BlendFactor::SrcAlpha,
            dst_factor: grr::BlendFactor::OneMinusSrcAlpha,
            blend_op: grr::BlendOp::Add,
          },
          alpha: grr::BlendChannel {
            src_factor: grr::BlendFactor::One,
            dst_factor: grr::BlendFactor::One,
            blend_op: grr::BlendOp::Add,
          },
        }],
      });
      gl.bind_samplers(
        0,
        &[gl.create_sampler(grr::SamplerDesc {
          min_filter: grr::Filter::Linear,
          mag_filter: grr::Filter::Linear,
          mip_map: None,
          address: (
            grr::SamplerAddress::ClampEdge,
            grr::SamplerAddress::ClampEdge,
            grr::SamplerAddress::ClampEdge,
          ),
          lod_bias: 0.0,
          lod: 0.0..10.0,
          compare: None,
          border_color: [0.0, 0.0, 0.0, 1.0],
        })?],
      );
      info!(
        "Created renderer: {}",
        CStr::from_ptr(gl.context().GetString(0x1F01) as _).to_str()?
      );

      Ok(Self { context, gl })
    }
  }

  pub fn resize(&self, size: PhysicalSize<u32>) {
    unsafe {
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
    }
  }

  pub fn clear(&self) {
    unsafe {
      self.gl.clear_attachment(
        grr::Framebuffer::DEFAULT,
        grr::ClearAttachment::ColorFloat(0, [0.0, 0.0, 0.0, 1.0]),
      );
      self
        .gl
        .clear_attachment(grr::Framebuffer::DEFAULT, grr::ClearAttachment::Depth(1.0));
    }
  }
}

pub struct Shader(grr::Pipeline);

impl Shader {
  pub fn new(renderer: &Renderer, vert_path: &str, frag_path: &str) -> Result<Self> {
    unsafe {
      let vert = renderer.gl.create_shader(
        grr::ShaderStage::Vertex,
        fs::read_to_string(vert_path)?.as_bytes(),
        grr::ShaderFlags::VERBOSE,
      )?;
      let frag = renderer.gl.create_shader(
        grr::ShaderStage::Fragment,
        fs::read_to_string(frag_path)?.as_bytes(),
        grr::ShaderFlags::VERBOSE,
      )?;
      Ok(Self(renderer.gl.create_graphics_pipeline(
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

  pub fn bind(&self, renderer: &Renderer) {
    unsafe {
      renderer.gl.bind_pipeline(self.0);
    }
  }

  pub fn set_mat4(&self, renderer: &Renderer, i: u32, val: &Mat4) {
    unsafe {
      renderer.gl.bind_uniform_constants(
        self.0,
        i,
        &[grr::Constant::Mat4x4(val.to_cols_array_2d())],
      );
    }
  }

  pub fn set_vec3(&self, renderer: &Renderer, i: u32, val: &Vec3) {
    unsafe {
      renderer
        .gl
        .bind_uniform_constants(self.0, i, &[grr::Constant::Vec3(val.to_array())]);
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
  vert_arr: grr::VertexArray,
  vert_buf: grr::Buffer,
  idx_buf: grr::Buffer,
  len: u32,
}

impl Mesh {
  pub fn new(renderer: &Renderer, vertices: Vec<Vertex>, indices: Vec<u16>) -> Result<Self> {
    unsafe {
      Ok(Self {
        vert_arr: renderer.gl.create_vertex_array(&Vertex::ATTR)?,
        vert_buf: renderer
          .gl
          .create_buffer_from_host(bytemuck::cast_slice(&vertices), grr::MemoryFlags::empty())?,
        idx_buf: renderer
          .gl
          .create_buffer_from_host(bytemuck::cast_slice(&indices), grr::MemoryFlags::empty())?,
        len: indices.len() as u32,
      })
    }
  }

  pub fn load(renderer: &Renderer, path: &str) -> Result<Self> {
    let obj: Obj<TexturedVertex> = obj::load_obj(BufReader::new(File::open(path)?))?;
    Self::new(
      renderer,
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

  pub fn draw(&self, renderer: &Renderer) {
    unsafe {
      renderer.gl.bind_vertex_array(self.vert_arr);
      renderer.gl.bind_vertex_buffers(
        self.vert_arr,
        0,
        &[grr::VertexBufferView {
          buffer: self.vert_buf,
          offset: 0,
          stride: Vertex::STRIDE,
          input_rate: grr::InputRate::Vertex,
        }],
      );
      renderer.gl.bind_index_buffer(self.vert_arr, self.idx_buf);
      renderer.gl.draw_indexed(
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
  pub fn new(renderer: &Renderer, data: &[u8], width: u32, height: u32) -> Result<Self> {
    unsafe {
      let (tex, view) = renderer.gl.create_image_and_view(
        grr::ImageType::D2 {
          width,
          height,
          layers: 1,
          samples: 1,
        },
        grr::Format::R8G8B8A8_SRGB,
        1,
      )?;
      renderer.gl.copy_host_to_image(
        data,
        tex,
        grr::HostImageCopy {
          host_layout: grr::MemoryLayout {
            base_format: grr::BaseFormat::RGBA,
            format_layout: grr::FormatLayout::U8,
            row_length: width,
            image_height: height,
            alignment: 4,
          },
          image_subresource: grr::SubresourceLayers {
            level: 0,
            layers: 0..1,
          },
          image_offset: grr::Offset { x: 0, y: 0, z: 0 },
          image_extent: grr::Extent {
            width,
            height,
            depth: 1,
          },
        },
      );
      Ok(Self(view))
    }
  }

  pub fn load(renderer: &Renderer, path: &str) -> Result<Self> {
    let img = image::open(path)?.to_rgba8();
    Self::new(renderer, img.as_raw(), img.width(), img.height())
  }

  pub fn bind(&self, renderer: &Renderer) {
    unsafe {
      renderer.gl.bind_image_views(0, &[self.0]);
    }
  }
}
