use std::collections::HashMap;
use std::rc::Rc;
use std::ops::Deref;
use std::any::Any;
use log::{error, trace};
use linkme::distributed_slice;
use serde::{Serialize, Deserialize, Deserializer};
use crate::ecs::World;
use crate::{Result, TypeIdNamed, WORLD};

pub struct AssetLoader {
  pub id: TypeIdNamed,
  pub loader: fn(&mut World, &str) -> Result<Rc<dyn Any>>,
}

#[distributed_slice]
pub static ASSET_LOADERS: [AssetLoader] = [..];

pub struct Assets {
  pub handles: HashMap<TypeIdNamed, Vec<Handle<dyn Any>>>,
}

impl Assets {
  pub fn new() -> Self {
    Self {
      handles: HashMap::new(),
    }
  }

  pub fn load<T: Any>(&mut self, path: &str) -> Result<Handle<T>> {
    let t = TypeIdNamed::of::<T>();
    let loader = match ASSET_LOADERS.iter().find(|l| l.id == t) {
      Some(s) => s,
      None => {
        error!("Unknown asset type '{}'.", t.name);
        panic!();
      }
    };
    self.handles.entry(t).or_insert(vec![]);
    let v = self.handles.get_mut(&t).unwrap();
    Ok(match v.iter().find(|h| h.name == path) {
      Some(h) => h.downcast(),
      None => {
        trace!("Loading '{}' from '{}'.", t.name, path);
        let h = Handle {
          name: path.to_string(),
          data: (loader.loader)(
            unsafe { WORLD.get_mut().unwrap() },
            &format!("assets/{}", path),
          )?,
        };
        v.push(h.clone());
        h.downcast()
      }
    })
  }

  pub fn get<T: Any>(&self) -> Vec<Handle<T>> {
    match self.handles.get(&TypeIdNamed::of::<T>()) {
      Some(l) => l.iter().map(|h| h.downcast()).collect(),
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
