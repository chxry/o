use std::fs;
use glutin::ContextBuilder;
use glutin::window::WindowBuilder;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{EventLoop, ControlFlow};
use log::LevelFilter;
use anyhow::Result;

const VERTICES: [f32; 15] = [
  -0.5, -0.5, 1.0, 0.0, 0.0, 0.5, -0.5, 0.0, 1.0, 0.0, 0.0, 0.5, 0.0, 0.0, 1.0,
];

fn main() -> Result<()> {
  env_logger::builder().filter_level(LevelFilter::Info).init();

  let event_loop = EventLoop::new();
  unsafe {
    let context = ContextBuilder::new()
      .build_windowed(WindowBuilder::new(), &event_loop)?
      .make_current()
      .unwrap();
    let gl = grr::Device::new(|s| context.get_proc_address(s), grr::Debug::Disable);

    let shader = Shader::new(&gl, "res/shader.vert", "res/shader.frag")?;
    let mesh = Mesh::new(&gl, &VERTICES)?;

    event_loop.run(move |event, _, control_flow| {
      *control_flow = ControlFlow::Wait;

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
          gl.clear_attachment(
            grr::Framebuffer::DEFAULT,
            grr::ClearAttachment::ColorFloat(0, [0.0, 0.0, 0.0, 1.0]),
          );

          shader.bind(&gl);
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
}

struct Mesh {
  arr: grr::VertexArray,
  buf: grr::Buffer,
  len: u32,
}

impl Mesh {
  unsafe fn new(gl: &grr::Device, verts: &[f32]) -> Result<Self> {
    Ok(Self {
      arr: gl.create_vertex_array(&[
        grr::VertexAttributeDesc {
          location: 0,
          binding: 0,
          format: grr::VertexFormat::Xy32Float,
          offset: 0,
        },
        grr::VertexAttributeDesc {
          location: 1,
          binding: 0,
          format: grr::VertexFormat::Xyz32Float,
          offset: 8,
        },
      ])?,
      buf: gl.create_buffer_from_host(grr::as_u8_slice(verts), grr::MemoryFlags::empty())?,
      len: verts.len() as u32,
    })
  }

  unsafe fn draw(&self, gl: &grr::Device) {
    gl.bind_vertex_array(self.arr);
    gl.bind_vertex_buffers(
      self.arr,
      0,
      &[grr::VertexBufferView {
        buffer: self.buf,
        offset: 0,
        stride: 20,
        input_rate: grr::InputRate::Vertex,
      }],
    );
    gl.draw(grr::Primitive::Triangles, 0..self.len, 0..1);
  }
}
