use std::collections::HashMap;
use std::any::{Any, TypeId};
use glutin::event::Event;
use log::error;
use anyhow::Result;
use crate::HashMapExt;

pub type System = &'static dyn Fn(&mut World) -> Result<()>;
pub type EventHandler = &'static dyn Fn(&mut World, &Event<()>) -> Result<()>;

#[derive(Hash, Eq, PartialEq)]
pub enum Stage {
  Start,
  PreDraw,
  Draw,
  PostDraw,
}

pub struct World {
  components: HashMap<TypeId, Vec<(usize, Box<dyn Any>)>>,
  resources: HashMap<TypeId, Box<dyn Any>>,
  systems: HashMap<Stage, Vec<System>>,
  event_handlers: Vec<EventHandler>,
}

impl World {
  pub fn new() -> Self {
    Self {
      components: HashMap::new(),
      resources: HashMap::new(),
      systems: HashMap::new(),
      event_handlers: Vec::new(),
    }
  }

  pub fn spawn(&self) -> Entity {
    unsafe {
      static mut COUNTER: usize = 0;
      COUNTER += 1;
      Entity {
        id: COUNTER,
        world: mutate(self),
      }
    }
  }

  pub fn query<T: Any>(&self) -> Vec<(Entity, &mut T)> {
    match self.components.get(&TypeId::of::<T>()) {
      Some(v) => v
        .iter()
        .map(|(e, b)| {
          (
            Entity {
              id: *e,
              world: mutate(self),
            },
            mutate(b.downcast_ref().unwrap()),
          )
        })
        .collect(),
      None => vec![],
    }
  }

  pub fn add_resource<T: Any>(&mut self, resource: T) {
    self.resources.insert(TypeId::of::<T>(), Box::new(resource));
  }

  pub fn get_resource<T: Any>(&self) -> Option<&mut T> {
    match self.resources.get(&TypeId::of::<T>()) {
      Some(r) => Some(mutate(r.downcast_ref().unwrap())),
      None => None,
    }
  }

  pub fn take_resource<T: Any>(&mut self) -> Option<T> {
    self
      .resources
      .remove(&TypeId::of::<T>())
      .map(|r| *r.downcast().unwrap())
  }

  pub fn add_system(&mut self, stage: Stage, sys: System) {
    self.systems.push_or_insert(stage, sys);
  }

  pub fn run_system(&self, stage: Stage) {
    if let Some(vec) = self.systems.get(&stage) {
      for sys in vec.clone() {
        if let Err(e) = sys(mutate(self)) {
          error!("Error in system: {}", e);
        }
      }
    }
  }

  pub fn add_event_handler(&mut self, handler: EventHandler) {
    self.event_handlers.push(handler);
  }

  pub fn run_event_handler(&self, event: &Event<()>) {
    for handler in self.event_handlers.clone() {
      if let Err(e) = handler(mutate(self), event) {
        error!("Error in event handler: {}", e);
      }
    }
  }
}

pub struct Entity<'w> {
  pub id: usize,
  world: &'w mut World,
}

impl Entity<'_> {
  pub fn insert<T: Any>(self, component: T) -> Self {
    self
      .world
      .components
      .push_or_insert(TypeId::of::<T>(), (self.id, Box::new(component)));
    self
  }

  pub fn get<T: Any>(&self) -> Option<&mut T> {
    match self.world.components.get(&TypeId::of::<T>()) {
      Some(v) => match v.iter().find(|(e, _)| *e == self.id) {
        Some(s) => Some(mutate(s.1.downcast_ref().unwrap())),
        None => None,
      },
      None => None,
    }
  }
}

// not very safe, use refcell or something
fn mutate<T>(t: &T) -> &mut T {
  unsafe { &mut *(t as *const T as *mut T) }
}
