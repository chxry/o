use std::collections::{HashMap, BTreeMap};
use std::any::Any;
use log::error;
use serde::{Serialize, Deserialize};
use crate::{Result, HashMapExt, TypeIdNamed, mutate, component, WORLD};

pub type System = &'static dyn Fn(&mut World) -> Result;

pub mod stage {
  pub const START: usize = 0;
  pub const PRE_DRAW: usize = 1;
  pub const DRAW: usize = 2;
  pub const POST_DRAW: usize = 3;
  pub const EVENT: usize = 4;
}

pub struct World {
  pub components: HashMap<TypeIdNamed, Vec<(usize, Box<dyn Any>)>>,
  pub resources: HashMap<TypeIdNamed, Box<dyn Any>>,
  systems: HashMap<usize, Vec<System>>,
}

impl World {
  pub fn new() -> Self {
    Self {
      components: HashMap::new(),
      resources: HashMap::new(),
      systems: HashMap::new(),
    }
  }

  pub fn spawn(&self, name: &str) -> Entity {
    self.spawn_empty().insert(Name(name.to_string()))
  }

  pub(crate) fn spawn_empty(&self) -> Entity {
    unsafe {
      static mut COUNTER: usize = 0;
      COUNTER += 1;
      Entity { id: COUNTER }
    }
  }

  pub fn query<T: Any>(&self) -> Vec<(Entity, &mut T)> {
    match self.components.get(&TypeIdNamed::of::<T>()) {
      Some(v) => v
        .iter()
        .map(|(e, b)| (Entity { id: *e }, mutate(b.downcast_ref().unwrap())))
        .collect(),
      None => vec![],
    }
  }

  pub fn get_id<T: Any>(&self, id: usize) -> Option<(Entity, &mut T)> {
    match self.components.get(&TypeIdNamed::of::<T>()) {
      Some(v) => v
        .iter()
        .find(|(e, _)| *e == id)
        .map(|s| (Entity { id: s.0 }, mutate(s.1.downcast_ref().unwrap()))),
      None => None,
    }
  }

  pub fn get_all(&self, id: usize) -> BTreeMap<TypeIdNamed, Vec<&mut Box<dyn Any>>> {
    let mut components = BTreeMap::new();
    for (t, v) in self.components.iter() {
      let v: Vec<&mut Box<dyn Any>> = v
        .iter()
        .filter(|i| i.0 == id)
        .map(|c| mutate(&c.1))
        .collect();
      if !v.is_empty() {
        components.insert(*t, v);
      }
    }
    components
  }

  pub fn get_name(&self, name: &str) -> Option<Entity> {
    self
      .query::<Name>()
      .into_iter()
      .find(|f| f.1 .0 == name)
      .map(|m| m.0)
  }

  pub fn remove_id(&self, t: TypeIdNamed, id: usize) {
    if let Some(v) = self.components.get(&t) {
      mutate(v).retain(|c| c.0 != id);
    }
  }

  pub fn add_resource<T: Any>(&self, resource: T) {
    mutate(self)
      .resources
      .insert(TypeIdNamed::of::<T>(), Box::new(resource));
  }

  pub fn get_resource<T: Any>(&self) -> Option<&mut T> {
    match self.resources.get(&TypeIdNamed::of::<T>()) {
      Some(r) => Some(mutate(r.downcast_ref().unwrap())),
      None => None,
    }
  }

  pub fn take_resource<T: Any>(&mut self) -> Option<T> {
    self
      .resources
      .remove(&TypeIdNamed::of::<T>())
      .map(|r| *r.downcast().unwrap())
  }

  pub fn add_system(&mut self, stage: usize, sys: System) {
    self.systems.push_or_insert(stage, sys);
  }

  pub fn run_system(&self, stage: usize) {
    if let Some(vec) = self.systems.get(&stage) {
      for sys in vec.clone() {
        if let Err(e) = sys(mutate(self)) {
          error!("Error in system: {}", e);
        }
      }
    }
  }
}

#[derive(Serialize, Deserialize)]
#[component]
pub struct Name(pub String);

pub struct Entity {
  pub id: usize,
}

// move stuff into here
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

  pub fn get<T: Any>(&self) -> Option<&mut T> {
    unsafe { WORLD.get().unwrap().get_id(self.id).map(|c| c.1) }
  }
}
