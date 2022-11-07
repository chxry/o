pub mod gfx;
pub mod ecs;

use std::collections::HashMap;
use std::hash::Hash;
use std::any::Any;
use std::error::Error;
use winit::event_loop::{EventLoop, ControlFlow};
use winit::event::WindowEvent;
use crate::gfx::Renderer;
use crate::ecs::{World, Stage, System, EventHandler};

pub use glam as math;
pub use log;
pub use grr;
pub use winit::event::Event;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

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
    self.world.add_resource(Renderer::new(&event_loop)?);

    self.world.run_system(Stage::Start);
    event_loop.run(move |event, _, control_flow| {
      let renderer = self.world.get_resource::<Renderer>().unwrap();
      renderer.window.request_redraw();
      self.world.run_event_handler(&event);
      match event {
        Event::WindowEvent {
          event: WindowEvent::CloseRequested,
          ..
        } => *control_flow = ControlFlow::Exit,
        Event::RedrawRequested(_) => {
          // automate this
          self.world.run_system(Stage::PreDraw);
          unsafe {
            renderer.gl.bind_framebuffer(grr::Framebuffer::DEFAULT);
          }
          renderer.resize(renderer.window.inner_size().into());
          renderer.clear(grr::Framebuffer::DEFAULT);
          self.world.run_system(Stage::Draw);
          self.world.run_system(Stage::PostDraw);

          renderer.context.swap_buffers();
        }
        _ => {}
      }
    })
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

// not very safe, use refcell or something
pub fn mutate<T>(t: &T) -> &mut T {
  unsafe { &mut *(t as *const T as *mut T) }
}
