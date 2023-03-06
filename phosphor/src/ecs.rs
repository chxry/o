use std::collections::{HashMap, BTreeMap};
use std::any::{Any, type_name};
use log::{error, trace};
use serde::{Serialize, Deserialize};
use crate::{Result, HashMapExt, TypeIdNamed, component, WORLD};

pub trait System = Fn(&mut World) -> Result;

pub mod stage {
  pub const INIT: usize = 0;
  pub const START: usize = 1;
  pub const PRE_DRAW: usize = 2;
  pub const DRAW: usize = 3;
  pub const POST_DRAW: usize = 4;
  pub const EVENT: usize = 5;
}

pub struct World {
  pub components: HashMap<TypeIdNamed, Vec<(usize, Box<dyn Any>)>>,
  resources: HashMap<TypeIdNamed, Box<dyn Any>>,
  systems: HashMap<usize, Vec<(&'static dyn System, &'static str)>>,
  count: usize,
}

impl World {
  pub fn new() -> Self {
    Self {
      components: HashMap::new(),
      resources: HashMap::new(),
      systems: HashMap::new(),
      count: 0,
    }
  }

  fn g(&self) -> &'static mut Self {
    unsafe { WORLD.get_mut().unwrap() }
  }

  pub fn spawn(&self, name: &str) -> Entity {
    self.spawn_empty().insert(Name(name.to_string()))
  }

  pub(crate) fn spawn_empty(&self) -> Entity {
    self.g().count += 1;
    Entity { id: self.g().count }
  }

  pub fn query<T: Any>(&self) -> Vec<(Entity, &mut T)> {
    match self.g().components.get_mut(&TypeIdNamed::of::<T>()) {
      Some(v) => v
        .iter_mut()
        .map(|(e, b)| (Entity { id: *e }, b.downcast_mut().unwrap()))
        .collect(),
      None => vec![],
    }
  }

  pub fn get_name(&self, name: &str) -> Option<Entity> {
    self
      .query::<Name>()
      .iter_mut()
      .find(|f| f.1 .0 == name)
      .map(|m| m.0)
  }

  //fix
  pub fn remove_id(&self, t: TypeIdNamed, id: usize) {
    if let Some(v) = self.g().components.get_mut(&t) {
      v.retain(|c| c.0 != id);
    }
  }

  pub fn add_resource<T: Any>(&self, resource: T) {
    self
      .g()
      .resources
      .insert(TypeIdNamed::of::<T>(), Box::new(resource));
  }

  pub fn get_resource<T: Any>(&self) -> Option<&mut T> {
    match self.g().resources.get_mut(&TypeIdNamed::of::<T>()) {
      Some(r) => Some(r.downcast_mut().unwrap()),
      None => None,
    }
  }

  pub fn take_resource<T: Any>(&mut self) -> Option<T> {
    self
      .resources
      .remove(&TypeIdNamed::of::<T>())
      .map(|r| *r.downcast().unwrap())
  }

  pub fn add_system<S: System + 'static>(&mut self, stage: usize, sys: S) {
    self
      .systems
      .push_or_insert(stage, (Box::leak(Box::new(sys)), type_name::<S>()));
  }

  pub fn run_system(&self, stage: usize) {
    if let Some(vec) = self.systems.get(&stage) {
      for (sys, name) in vec.clone() {
        trace!("Running system '{}'.", name);
        if let Err(e) = sys(self.g()) {
          error!("Error in system '{}': {}", name, e);
        }
      }
    }
  }
}

#[derive(Serialize, Deserialize)]
#[component]
pub struct Name(pub String);

#[derive(Clone, Copy)]
pub struct Entity {
  pub id: usize,
}

impl Entity {
  pub fn insert<T: Any>(self, component: T) -> Self {
    unsafe {
      WORLD
        .get_mut()
        .unwrap()
        .components
        .push_or_insert(TypeIdNamed::of::<T>(), (self.id, Box::new(component)));
    }
    self
  }

  pub fn get<T: Any>(&self) -> Vec<&mut T> {
    unsafe {
      match WORLD
        .get_mut()
        .unwrap()
        .components
        .get_mut(&TypeIdNamed::of::<T>())
      {
        Some(v) => v
          .iter_mut()
          .filter_map(|(e, c)| (*e == self.id).then(|| c.downcast_mut().unwrap()))
          .collect(),
        None => vec![],
      }
    }
  }

  pub fn get_one<T: Any>(&self) -> Option<&mut T> {
    self.get().pop()
  }

  pub fn get_all(&self) -> BTreeMap<TypeIdNamed, Vec<&mut Box<dyn Any>>> {
    let mut components = BTreeMap::new();
    unsafe {
      for (t, v) in WORLD.get_mut().unwrap().components.iter_mut() {
        let v: Vec<&mut Box<dyn Any>> = v
          .iter_mut()
          .filter_map(|(e, c)| (*e == self.id).then(|| c))
          .collect();
        if !v.is_empty() {
          components.insert(*t, v);
        }
      }
    }
    components
  }
}
