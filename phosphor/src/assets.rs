use std::rc::Rc;
use std::ops::Deref;
use std::collections::HashMap;
use std::any::Any;
use std::path::Path;
use std::io::BufReader;
use std::fs::File;
use std::fmt::Display;
use obj::{Obj, TexturedVertex};
use image::imageops::flip_vertical_in_place;
use crate::gfx::{Texture, Mesh, Vertex};
use crate::{Result, HashMapExt, TypeIdNamed};

pub struct Assets {
  pub handles: HashMap<TypeIdNamed, Vec<Handle<dyn Any>>>,
}

impl Assets {
  pub fn new() -> Self {
    Self {
      handles: HashMap::new(),
    }
  }

  pub fn get<T: 'static>(&self) -> Vec<Handle<T>> {
    match self.handles.get(&TypeIdNamed::of::<T>()) {
      Some(v) => v.iter().map(|h| h.downcast()).collect(),
      None => vec![],
    }
  }

  pub fn load_tex<P: AsRef<Path> + Display + Clone>(&mut self, path: P) -> Result<Handle<Texture>> {
    // check if already inserted and just return that handle
    let mut img = image::open(path.clone())?.to_rgba8();
    flip_vertical_in_place(&mut img);
    let h = Handle::new(
      path.to_string(),
      Texture::new(img.as_raw(), img.width(), img.height()),
    );
    self
      .handles
      .push_or_insert(TypeIdNamed::of::<Texture>(), h.any());
    Ok(h)
  }

  pub fn load_mesh<P: AsRef<Path> + Display + Clone>(&mut self, path: P) -> Result<Handle<Mesh>> {
    let obj: Obj<TexturedVertex> = obj::load_obj(BufReader::new(File::open(path.clone())?))?;
    let h = Handle::new(
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
    );
    self
      .handles
      .push_or_insert(TypeIdNamed::of::<Mesh>(), h.any());
    Ok(h)
  }
}

#[derive(Clone)]
pub struct Handle<T: ?Sized> {
  pub name: String,
  data: Rc<T>,
}

impl<T> Deref for Handle<T> {
  type Target = T;

  fn deref(&self) -> &T {
    &self.data
  }
}

impl<T: 'static> Handle<T> {
  fn new(name: String, data: T) -> Self {
    Self {
      name,
      data: Rc::new(data),
    }
  }

  fn any(&self) -> Handle<dyn Any> {
    Handle {
      name: self.name.clone(),
      data: self.data.clone() as _,
    }
  }
}

impl Handle<dyn Any> {
  fn downcast<T: 'static>(&self) -> Handle<T> {
    Handle {
      name: self.name.clone(),
      data: self.data.clone().downcast().unwrap(),
    }
  }
}
