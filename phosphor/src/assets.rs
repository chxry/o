use std::rc::Rc;
use std::ops::Deref;
use std::collections::HashMap;
use std::any::Any;
use std::io::BufReader;
use std::fs::File;
use obj::{Obj, TexturedVertex};
use image::imageops::flip_vertical_in_place;
use log::{error, trace};
use serde::{Serialize, Deserialize, Deserializer};
use crate::gfx::{Texture, Mesh, Vertex};
use crate::{Result, TypeIdNamed, WORLD};

pub struct AssetLoader {
  loader: &'static dyn Fn(&str) -> Result<Rc<dyn Any>>,
  pub handles: Vec<Handle<dyn Any>>,
}

pub struct Assets {
  pub handles: HashMap<TypeIdNamed, AssetLoader>,
}

fn load_tex(path: &str) -> Result<Rc<dyn Any>> {
  let mut img = image::open(path)?.to_rgba8();
  flip_vertical_in_place(&mut img);
  Ok(Rc::new(Texture::new(
    img.as_raw(),
    img.width(),
    img.height(),
  )))
}

fn load_mesh(path: &str) -> Result<Rc<dyn Any>> {
  let obj: Obj<TexturedVertex> = obj::load_obj(BufReader::new(File::open(path)?))?;
  Ok(Rc::new(Mesh::new(
    &obj
      .vertices
      .iter()
      .map(|v| Vertex {
        pos: v.position,
        uv: [v.texture[0], v.texture[1]],
        normal: v.normal,
      })
      .collect::<Vec<_>>(),
    &obj.indices,
  )))
}

impl Assets {
  pub fn new() -> Self {
    Self {
      handles: HashMap::from([
        (
          TypeIdNamed::of::<Texture>(),
          AssetLoader {
            loader: &load_tex,
            handles: vec![],
          },
        ),
        (
          TypeIdNamed::of::<Mesh>(),
          AssetLoader {
            loader: &load_mesh,
            handles: vec![],
          },
        ),
      ]),
    }
  }

  pub fn load<T: Any>(&mut self, path: &str) -> Result<Handle<T>> {
    let t = TypeIdNamed::of::<T>();
    let loader = match self.handles.get_mut(&t) {
      Some(s) => s,
      None => {
        error!("Unknown asset type '{}'.", t.name);
        panic!();
      }
    };
    trace!("Loading '{}' from '{}'.", t.name, path);
    Ok(match loader.handles.iter().find(|h| h.name == path) {
      Some(h) => h.downcast(),
      None => {
        let h = Handle {
          name: path.to_string(),
          data: (loader.loader)(&format!("assets/{}", path)).unwrap(),
        };
        loader.handles.push(h.clone());
        h.downcast()
      }
    })
  }

  pub fn get<T: Any>(&self) -> Vec<Handle<T>> {
    match self.handles.get(&TypeIdNamed::of::<T>()) {
      Some(l) => l.handles.iter().map(|h| h.downcast()).collect(),
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
  pub fn downcast<T: Any>(&self) -> Handle<T> {
    Handle {
      name: self.name.clone(),
      data: self.data.clone().downcast().unwrap(),
    }
  }
}
