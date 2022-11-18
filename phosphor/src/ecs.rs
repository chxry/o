use std::collections::HashMap;
use std::any::{Any, TypeId};
use glfw::WindowEvent;
use log::error;
use crate::{Result, HashMapExt, mutate};

pub type System = &'static dyn Fn(&mut World) -> Result;
pub type EventHandler = &'static dyn Fn(&mut World, &WindowEvent) -> Result<()>;

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
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

  pub fn spawn(&self, name: &str) -> Entity {
    unsafe {
      static mut COUNTER: usize = 0;
      COUNTER += 1;
      Entity {
        id: COUNTER,
        world: mutate(self),
      }
      .insert(Name(name.to_string()))
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

  pub fn get_id<T: Any>(&self, id: usize) -> Option<(Entity, &mut T)> {
    match self.components.get(&TypeId::of::<T>()) {
      Some(v) => v.iter().find(|(e, _)| *e == id).map(|s| {
        (
          Entity {
            id: s.0,
            world: mutate(self),
          },
          mutate(s.1.downcast_ref().unwrap()),
        )
      }),
      None => None,
    }
  }

  pub fn get_name(&self, name: &'static str) -> Option<Entity> {
    self
      .query::<Name>()
      .into_iter()
      .find(|f| f.1 .0 == name)
      .map(|m| m.0)
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

  pub fn run_event_handler(&self, event: WindowEvent) {
    for handler in self.event_handlers.clone() {
      if let Err(e) = handler(mutate(self), &event) {
        error!("Error in event handler: {}", e);
      }
    }
  }
}

pub struct Name(pub String);

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
    self.world.get_id(self.id).map(|c| c.1)
  }
}
