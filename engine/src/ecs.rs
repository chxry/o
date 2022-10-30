use std::collections::HashMap;
use std::any::{Any, TypeId};
use glutin::event::Event;
use log::error;
use anyhow::Result;
use crate::gfx::Renderer;
use crate::{HashMapExt, mutate};

pub type System = &'static dyn Fn(Context) -> Result<()>;
pub type EventHandler = &'static dyn Fn(Context, &Event<()>) -> Result<()>;

pub struct Context<'a> {
  pub world: &'a mut World,
  pub renderer: &'a Renderer,
}

#[derive(Hash, Eq, PartialEq)]
pub enum Stage {
  Start,
  Draw,
  PostDraw,
}

#[derive(PartialEq, Debug)]
pub struct Entity(usize);

pub struct World {
  components: HashMap<TypeId, Vec<(Entity, Box<dyn Any>)>>,
  resources: HashMap<TypeId, Box<dyn Any>>,
  systems: HashMap<Stage, Vec<System>>,
  event_handlers: Vec<EventHandler>,
  counter: usize,
}

impl World {
  pub fn new() -> Self {
    Self {
      components: HashMap::new(),
      resources: HashMap::new(),
      systems: HashMap::new(),
      event_handlers: Vec::new(),
      counter: 0,
    }
  }

  pub fn spawn(&mut self) -> Entity {
    self.counter += 1;
    Entity(self.counter)
  }

  pub fn insert<T: Any>(&mut self, entity: &Entity, component: T) {
    self
      .components
      .push_or_insert(TypeId::of::<T>(), (Entity(entity.0), Box::new(component)));
  }

  pub fn query<T: Any>(&self) -> Vec<(&Entity, &T)> {
    match self.components.get(&TypeId::of::<T>()) {
      Some(v) => v
        .iter()
        .map(|(e, b)| (e, b.downcast_ref().unwrap()))
        .collect(),
      None => vec![],
    }
  }

  pub fn get<T: Any>(&self, entity: &Entity) -> Option<&T> {
    match self.components.get(&TypeId::of::<T>()) {
      Some(v) => match v.iter().find(|(e, _)| e == entity) {
        Some(s) => s.1.downcast_ref(),
        None => None,
      },
      None => None,
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

  pub fn add_system(&mut self, stage: Stage, sys: System) {
    self.systems.push_or_insert(stage, sys);
  }

  pub fn run_system(&mut self, renderer: &Renderer, stage: Stage) {
    if let Some(vec) = self.systems.get(&stage) {
      for sys in vec.clone() {
        if let Err(e) = sys(Context {
          world: self,
          renderer,
        }) {
          error!("Error in system: {}", e);
        }
      }
    }
  }

  pub fn add_event_handler(&mut self, handler: EventHandler) {
    self.event_handlers.push(handler);
  }

  pub fn run_event_handler(&mut self, renderer: &Renderer, event: &Event<()>) {
    for handler in self.event_handlers.clone() {
      if let Err(e) = handler(
        Context {
          world: self,
          renderer,
        },
        event,
      ) {
        error!("Error in event handler: {}", e);
      }
    }
  }
}
