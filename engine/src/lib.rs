pub mod gfx;
pub mod ecs;
pub mod scene;
pub mod ui;

use std::collections::HashMap;
use std::hash::Hash;
use std::any::Any;
use glutin::event_loop::{EventLoop, ControlFlow};
use glutin::event::{Event, WindowEvent};
use anyhow::Result;
use crate::gfx::Renderer;
use crate::ecs::{World, Stage, System, EventHandler};

pub use glam as math;
pub use log;

pub struct Engine {
  world: World,
}

impl Engine {
  pub fn new() -> Self {
    Self {
      world: World::new(),
    }
  }

  pub fn add_resource<T: Any>(mut self, resource: T) -> Self {
    self.world.add_resource(resource);
    self
  }

  pub fn add_system(mut self, stage: Stage, sys: System) -> Self {
    self.world.add_system(stage, sys);
    self
  }

  pub fn add_event_handler(mut self, handler: EventHandler) -> Self {
    self.world.add_event_handler(handler);
    self
  }

  pub fn run(mut self) -> Result<()> {
    let event_loop = EventLoop::new();
    let renderer = Renderer::new(&event_loop)?;

    self.world.run_system(&renderer, Stage::Start);
    event_loop.run(move |event, _, control_flow| {
      renderer.context.window().request_redraw();
      self.world.run_event_handler(&renderer, &event);
      match event {
        Event::WindowEvent {
          event: WindowEvent::CloseRequested,
          ..
        } => *control_flow = ControlFlow::Exit,
        Event::RedrawRequested(_) => {
          renderer.resize(renderer.context.window().inner_size());
          renderer.clear();
          // automate this
          self.world.run_system(&renderer, Stage::PreDraw);
          self.world.run_system(&renderer, Stage::Draw);
          self.world.run_system(&renderer, Stage::PostDraw);

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

fn mutate<T>(t: &T) -> &mut T {
  unsafe { &mut *(t as *const T as *mut T) }
}
