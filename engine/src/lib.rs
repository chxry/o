pub mod ecs;
pub mod gfx;

use std::collections::HashMap;
use std::hash::Hash;
use glutin::event_loop::{EventLoop, ControlFlow};
use glutin::event::{Event, WindowEvent};
use anyhow::Result;
use crate::gfx::Renderer;
use crate::ecs::{System, Stage, World};

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

  fn run_system(&mut self, stage: Stage) {
    if let Some(vec) = self.systems.get(&stage) {
      for sys in vec {
        sys(&mut self.world);
      }
    }
  }

  pub fn run(mut self) -> Result<()> {
    let event_loop = EventLoop::new();
    let renderer = Renderer::new(&event_loop)?;

    self.run_system(Stage::Start);
    event_loop.run(move |event, _, control_flow| {
      renderer.context.window().request_redraw();
      match event {
        Event::WindowEvent { event, .. } => match event {
          WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
          WindowEvent::Resized(size) => renderer.resize(size),
          _ => {}
        },
        Event::RedrawRequested(_) => {
          renderer.clear();
          self.run_system(Stage::Draw);
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
