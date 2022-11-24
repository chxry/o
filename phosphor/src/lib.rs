pub mod gfx;
pub mod ecs;
pub mod assets;

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::any::{Any, TypeId, type_name};
use std::cmp::Ordering;
use glfw::Context;
use crate::gfx::Renderer;
use crate::ecs::{World, Stage, System};
use crate::assets::Assets;

pub use glam as math;
pub use log;
pub use glfw;

pub type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

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

  pub fn run(mut self) -> Result<()> {
    self.world.add_resource(Assets::new());
    self.world.add_resource(Renderer::new()?);
    let renderer = self.world.get_resource::<Renderer>().unwrap();
    self.world.run_system(Stage::Start);
    while !renderer.window.should_close() {
      renderer.glfw.poll_events();
      for (_, event) in renderer.events.try_iter() {
        self.world.run_event_handler(event);
      }
      self.world.run_system(Stage::PreDraw);
      self.world.run_system(Stage::Draw);
      self.world.run_system(Stage::PostDraw);
      renderer.window.swap_buffers();
    }
    Ok(())
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

#[derive(Clone, Copy, Eq)]
pub struct TypeIdNamed {
  pub id: TypeId,
  pub name: &'static str,
}

impl TypeIdNamed {
  pub fn of<T: Any>() -> Self {
    Self {
      id: TypeId::of::<T>(),
      name: type_name::<T>(),
    }
  }
}

impl Hash for TypeIdNamed {
  fn hash<H: Hasher>(&self, h: &mut H) {
    self.id.hash(h)
  }
}

impl PartialEq for TypeIdNamed {
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
  }
}

impl PartialOrd for TypeIdNamed {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for TypeIdNamed {
  fn cmp(&self, other: &Self) -> Ordering {
    self.id.cmp(&other.id)
  }
}

// not very safe, use refcell or something
pub fn mutate<T>(t: &T) -> &mut T {
  unsafe { &mut *(t as *const T as *mut T) }
}
