use std::rc::Rc;
use std::ops::Deref;
use std::collections::HashMap;
use std::any::Any;
use std::io::BufReader;
use std::fs::File;
use obj::{Obj, TexturedVertex};
use image::imageops::flip_vertical_in_place;
use log::error;
use serde::{Serialize, Deserialize, Deserializer};
use crate::gfx::{Texture, Mesh, Vertex};
use crate::{Result, TypeIdNamed, WORLD};

pub struct Assets {
  pub handles: HashMap<
    TypeIdNamed,
    (
      Vec<Handle<dyn Any>>,
      &'static dyn Fn(&str) -> Result<Handle<dyn Any>>,
    ),
  >,
}

fn load_tex(path: &str) -> Result<Handle<dyn Any>> {
  let mut img = image::open(path)?.to_rgba8();
  flip_vertical_in_place(&mut img);
  Ok(Handle::new(
    path.to_string(),
    Texture::new(img.as_raw(), img.width(), img.height()),
  ))
}

fn load_mesh(path: &str) -> Result<Handle<dyn Any>> {
  let obj: Obj<TexturedVertex> = obj::load_obj(BufReader::new(File::open(path)?))?;
  Ok(Handle::new(
    path.to_string(),
    Mesh::new(
      &obj
        .vertices
        .iter()
        .map(|v| Vertex {
          pos: v.position,
          uv: [v.texture[0], v.texture[1]],
        })
        .collect::<Vec<_>>(),
      &obj.indices,
    ),
  ))
}

impl Assets {
  pub fn new() -> Self {
    Self {
      handles: HashMap::from([
        (TypeIdNamed::of::<Texture>(), (vec![], &load_tex as _)),
        (TypeIdNamed::of::<Mesh>(), (vec![], &load_mesh as _)),
      ]),
    }
  }

  pub fn load<T: Any>(&mut self, path: &str) -> Result<Handle<T>> {
    let t = TypeIdNamed::of::<T>();
    let (v, l) = match self.handles.get_mut(&t) {
      Some(s) => s,
      None => {
        error!("Unknown asset type '{}'.", t.name);
        panic!();
      }
    };
    match v.iter().find(|h| h.name == path) {
      Some(h) => Ok(h.downcast()),
      None => (l)(path).map(|h| {
        v.push(h.clone());
        h.downcast()
      }),
    }
  }

  pub fn get<T: Any>(&self) -> Vec<Handle<T>> {
    match self.handles.get(&TypeIdNamed::of::<T>()) {
      Some(v) => v.0.iter().map(|h| h.downcast()).collect(),
      None => vec![],
    }
  }
}

#[derive(Serialize)]
pub struct Handle<T: ?Sized> {
  pub name: String,
  #[serde(skip)]
  data: Rc<T>,
}

impl<'de, T: Any> Deserialize<'de> for Handle<T> {
  fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
    let name: String = Deserialize::deserialize(deserializer)?;
    Ok(unsafe {
      WORLD
        .get_mut()
        .unwrap()
        .get_resource::<Assets>()
        .unwrap()
        .load(&name)
        .unwrap()
    })
  }
}

impl<T> Deref for Handle<T> {
  type Target = T;

  fn deref(&self) -> &T {
    &self.data
  }
}

impl<T: ?Sized> Clone for Handle<T> {
  fn clone(&self) -> Self {
    Self {
      name: self.name.clone(),
      data: self.data.clone(),
    }
  }
}

impl Handle<dyn Any> {
  fn new<T: Any>(name: String, data: T) -> Self {
    Self {
      name,
      data: Rc::new(data),
    }
  }

  pub fn downcast<T: Any>(&self) -> Handle<T> {
    Handle {
      name: self.name.clone(),
      data: self.data.clone().downcast().unwrap(),
    }
  }
}
