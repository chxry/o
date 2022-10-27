use std::collections::HashMap;
use std::any::{Any, TypeId};
use crate::HashMapExt;

pub type System = &'static dyn Fn(&mut World);

#[derive(Hash, Eq, PartialEq)]
pub enum Stage {
  Start,
  Draw,
}

#[derive(Clone, Debug)]
pub struct Entity(usize);

pub struct World {
  components: HashMap<TypeId, Vec<(Entity, Box<dyn Any>)>>,
  counter: usize,
}

impl World {
  pub fn new() -> Self {
    Self {
      components: HashMap::new(),
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
      .push_or_insert(TypeId::of::<T>(), (entity.clone(), Box::new(component)));
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
}
