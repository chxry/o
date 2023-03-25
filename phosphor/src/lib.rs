#![feature(const_type_id)]
#![feature(const_type_name)]
#![feature(trait_alias)]
#![allow(clippy::new_without_default)]
pub mod gfx;
pub mod ecs;
pub mod assets;
pub mod scene;

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::any::{Any, TypeId, type_name};
use std::cmp::Ordering;
use std::mem;
use glfw::Context;
use once_cell::unsync::OnceCell;
use crate::gfx::Renderer;
use crate::ecs::{World, System, stage};
use crate::assets::Assets;

pub use phosphor_derive::*;
pub use glam as math;
pub use log;
pub use glfw;
pub use bincode;
pub use linkme;

pub type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

static mut WORLD: OnceCell<World> = OnceCell::new();

pub struct Engine;
pub struct DeltaTime(pub f32);

impl Engine {
  pub fn new() -> Self {
    unsafe {
      let _ = WORLD.set(World::new());
    }
    Self
  }

  pub fn add_resource<T: Any>(self, resource: T) -> Self {
    unsafe {
      WORLD.get_mut().unwrap().add_resource(resource);
    }
    self
  }

  pub fn add_system<S: System + 'static>(self, stage: usize, sys: S) -> Self {
    unsafe {
      WORLD.get_mut().unwrap().add_system(stage, sys);
    }
    self
  }

  pub fn run(self) -> Result<()> {
    let world = unsafe { WORLD.get_mut().unwrap() };
    world.add_resource(Assets::new());
    world.add_resource(Renderer::new()?);
    let renderer = world.get_resource::<Renderer>().unwrap();
    world.run_system(stage::INIT);
    world.run_system(stage::START);
    let mut t = renderer.glfw.get_time();
    while !renderer.window.should_close() {
      puffin::GlobalProfiler::lock().new_frame();
      let n = renderer.glfw.get_time();
      world.add_resource(DeltaTime((n - t) as _));
      t = n;
      renderer.glfw.poll_events();
      for (_, event) in renderer.events.try_iter() {
        world.add_resource(event);
        world.run_system(stage::EVENT);
      }
      world.run_system(stage::PRE_DRAW);
      world.run_system(stage::DRAW);
      world.run_system(stage::POST_DRAW);
      renderer.window.swap_buffers();
    }
    Ok(())
  }
}

pub trait HashMapExt<K, V> {
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
  pub const fn of<T: Any>() -> Self {
    Self {
      id: TypeId::of::<T>(),
      name: type_name::<T>(),
    }
  }

  pub fn id(&self) -> usize {
    unsafe { mem::transmute(self.id) }
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
