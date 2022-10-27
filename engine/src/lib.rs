pub mod ecs;
pub mod gfx;

use std::collections::HashMap;
use std::hash::Hash;
use glutin::event_loop::{EventLoop, ControlFlow};
use glutin::event::{Event, WindowEvent};
use glam::{Mat4, Vec3};
use log::{error, warn};
use anyhow::Result;
use crate::gfx::{Renderer, Shader, Mesh, Texture};
use crate::ecs::{System, Stage, World, Context, Transform};

pub use glam as math;
pub use log;

pub struct Engine {
  world: World,
  systems: HashMap<Stage, Vec<System>>,
}

impl Engine {
  pub fn new() -> Self {
    Self {
      world: World::new(),
      systems: HashMap::new(),
    }
  }

  pub fn add_system(mut self, stage: Stage, sys: System) -> Self {
    self.systems.push_or_insert(stage, sys);
    self
  }

  fn run_system(&mut self, renderer: &Renderer, stage: Stage) {
    if let Some(vec) = self.systems.get(&stage) {
      for sys in vec {
        if let Err(e) = sys(Context {
          world: &mut self.world,
          renderer,
        }) {
          error!("Error in system: {}", e);
        }
      }
    }
  }

  pub fn run(mut self) -> Result<()> {
    let event_loop = EventLoop::new();
    let renderer = Renderer::new(&event_loop)?;
    let shader = Shader::new(&renderer, "res/shader.vert", "res/shader.frag")?;
    let tex = Texture::new(&renderer, "res/floppa.jpg")?;

    self.run_system(&renderer, Stage::Start);
    event_loop.run(move |event, _, control_flow| {
      renderer.context.window().request_redraw();
      match event {
        Event::WindowEvent { event, .. } => match event {
          WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
          WindowEvent::Resized(size) => renderer.resize(size),
          _ => {}
        },
        Event::RedrawRequested(_) => {
          let size = renderer.context.window().inner_size();
          renderer.clear();

          shader.bind(&renderer);
          tex.bind(&renderer);
          let view = Mat4::look_to_rh(Vec3::new(0.0, 1.0, -5.0), Vec3::Z, Vec3::Y);
          let projection =
            Mat4::perspective_rh_gl(0.8, size.width as f32 / size.height as f32, 0.1, 10.0);
          shader.bind(&renderer);
          shader.set_mat4(&renderer, 1, view);
          shader.set_mat4(&renderer, 2, projection);
          for (e, mesh) in self.world.query::<Mesh>() {
            match self.world.get::<Transform>(e) {
              Some(t) => {
                shader.set_mat4(&renderer, 0, t.as_mat4());
                mesh.draw(&renderer);
              }
              None => warn!("Mesh on {:?} will not be rendered without a Transform.", e),
            }
          }
          self.run_system(&renderer, Stage::Draw);
          renderer.context.swap_buffers().unwrap();
        }
        _ => {}
      }
    });
  }
}

trait HashMapExt<K, V> {
  fn push_or_insert(&mut self, key: K, val: V);
}

impl<K: Hash + Eq, V> HashMapExt<K, V> for HashMap<K, Vec<V>> {
  fn push_or_insert(&mut self, key: K, val: V) {
    match self.get_mut(&key) {
      Some(vec) => vec.push(val),
      None => {
        self.insert(key, vec![val]);
      }
    };
  }
}
