use std::fs::{self, File};
use std::io::BufReader;
use glutin::ContextBuilder;
use glutin::window::WindowBuilder;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{EventLoop, ControlFlow};
use glam::{Mat4, Vec3};
use obj::{Obj, TexturedVertex};
use log::LevelFilter;
use anyhow::Result;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
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

fn main() -> Result<()> {
  env_logger::builder().filter_level(LevelFilter::Info).init();

  let event_loop = EventLoop::new();
  unsafe {
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

    let shader = Shader::new(&gl, "res/shader.vert", "res/shader.frag")?;
    let mesh = Mesh::load(&gl, "res/suzanne.obj")?;
    let tex = Texture::new(&gl, "res/floppa.jpg")?;

    let mut rot = 0.0;
    event_loop.run(move |event, _, control_flow| {
      context.window().request_redraw();
      rot += 0.0001;
      match event {
        Event::WindowEvent { event, .. } => match event {
          WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
          WindowEvent::Resized(size) => {
            gl.set_viewport(
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
            gl.set_scissor(
              0,
              &[grr::Region {
                x: 0,
                y: 0,
                w: size.width as _,
                h: size.height as _,
              }],
            );
          }
          _ => {}
        },
        Event::RedrawRequested(_) => {
          let size = context.window().inner_size();
          gl.clear_attachment(
            grr::Framebuffer::DEFAULT,
            grr::ClearAttachment::ColorFloat(0, [0.0, 0.0, 0.0, 1.0]),
          );
          gl.clear_attachment(grr::Framebuffer::DEFAULT, grr::ClearAttachment::Depth(1.0));

          let model = Mat4::from_rotation_y(rot);
          let view = Mat4::from_translation(Vec3::new(0.0, 0.0, -3.0));
          let projection =
            Mat4::perspective_rh(0.8, size.width as f32 / size.height as f32, 0.1, 10.0);
          shader.bind(&gl);
          shader.set_mat4(&gl, 0, model);
          shader.set_mat4(&gl, 1, view);
          shader.set_mat4(&gl, 2, projection);
          tex.bind(&gl);
          mesh.draw(&gl);

          context.swap_buffers().unwrap();
        }
        _ => {}
      }
    })
  }
}

struct Shader(grr::Pipeline);

impl Shader {
  unsafe fn new(gl: &grr::Device, vert_path: &str, frag_path: &str) -> Result<Self> {
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

  unsafe fn bind(&self, gl: &grr::Device) {
    gl.bind_pipeline(self.0);
  }

  unsafe fn set_mat4(&self, gl: &grr::Device, i: u32, val: Mat4) {
    gl.bind_uniform_constants(self.0, i, &[grr::Constant::Mat4x4(val.to_cols_array_2d())]);
  }
}

struct Mesh {
  arr: grr::VertexArray,
  vert_buf: grr::Buffer,
  idx_buf: grr::Buffer,
  len: u32,
}

impl Mesh {
  unsafe fn new(gl: &grr::Device, vertices: Vec<Vertex>, indices: Vec<u16>) -> Result<Self> {
    Ok(Self {
      arr: gl.create_vertex_array(&Vertex::ATTR)?,
      vert_buf: gl
        .create_buffer_from_host(bytemuck::cast_slice(&vertices), grr::MemoryFlags::empty())?,
      idx_buf: gl
        .create_buffer_from_host(bytemuck::cast_slice(&indices), grr::MemoryFlags::empty())?,
      len: indices.len() as u32,
    })
  }

  unsafe fn load(gl: &grr::Device, path: &str) -> Result<Self> {
    let obj: Obj<TexturedVertex> = obj::load_obj(BufReader::new(File::open(path)?))?;
    Mesh::new(
      &gl,
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

  unsafe fn draw(&self, gl: &grr::Device) {
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

struct Texture(grr::ImageView);

impl Texture {
  unsafe fn new(gl: &grr::Device, path: &str) -> Result<Self> {
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
      &img.as_raw(),
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

  unsafe fn bind(&self, gl: &grr::Device) {
    gl.bind_image_views(0, &[self.0]);
  }
}
