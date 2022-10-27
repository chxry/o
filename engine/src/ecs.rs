use std::collections::HashMap;
use std::any::{Any, TypeId};
use glam::{Vec3, Quat, EulerRot, Mat4};
use anyhow::Result;
use crate::gfx::Renderer;
use crate::HashMapExt;

pub type System = &'static dyn Fn(Context) -> Result<()>;

pub struct Context<'a> {
  pub world: &'a mut World,
  pub renderer: &'a Renderer,
}

#[derive(Hash, Eq, PartialEq)]
pub enum Stage {
  Start,
  Draw,
}

#[derive(PartialEq, Debug)]
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
        Some(s) => Some(s.1.downcast_ref().unwrap()),
        None => None,
      },
      None => None,
    }
  }
}

pub struct Transform {
  pub position: Vec3,
  pub rotation: Quat,
  pub scale: Vec3,
}

impl Transform {
  pub fn new() -> Self {
    Self {
      position: Vec3::ZERO,
      rotation: Quat::IDENTITY,
      scale: Vec3::ONE,
    }
  }

  pub fn pos(mut self, position: Vec3) -> Self {
    self.position = position;
    self
  }

  pub fn rot_quat(mut self, rotation: Quat) -> Self {
    self.rotation = rotation;
    self
  }

  pub fn rot_euler(mut self, rotation: Vec3) -> Self {
    self.rotation = Quat::from_euler(EulerRot::XYZ, rotation.x, rotation.y, rotation.z);
    self
  }

  pub fn scale(mut self, scale: Vec3) -> Self {
    self.scale = scale;
    self
  }

  pub fn as_mat4(&self) -> Mat4 {
    Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
  }
}
