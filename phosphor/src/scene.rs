use std::collections::HashMap;
use std::fs::File;
use std::any::Any;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use linkme::distributed_slice;
use log::{info, warn, trace};
use crate::ecs::World;
use crate::assets::Assets;
use crate::{TypeIdNamed, Result, HashMapExt};

#[derive(Serialize, Deserialize)]
pub struct Scene {
  entities: HashMap<usize, Vec<(usize, Vec<u8>)>>,
}

pub struct Loader {
  pub id: TypeIdNamed,
  pub save: fn(&Box<dyn Any>) -> Vec<u8>,
  pub load: fn(Vec<u8>, &mut Assets) -> Box<dyn Any>,
}

#[distributed_slice]
pub static COMPONENT_LOADERS: [Loader] = [..];

impl Scene {
  pub fn save(world: &World, path: PathBuf) -> Result {
    let mut scene = Scene {
      entities: HashMap::new(),
    };
    for (t, v) in world.components.iter() {
      if let Some(loader) = COMPONENT_LOADERS.iter().find(|l| l.id == *t) {
        for (i, d) in v {
          trace!("Saving '{}' on {}.", t.name, i);
          scene
            .entities
            .push_or_insert(*i, (t.id(), (loader.save)(d)));
        }
      } else {
        warn!("{} cannot be serialized.", t.name);
      }
    }
    bincode::serialize_into(File::create(path.clone())?, &scene)?;
    info!("Saved scene to '{}'.", path.display());
    Ok(())
  }

  pub fn load(world: &mut World, path: PathBuf) -> Result {
    let scene: Scene = bincode::deserialize_from(File::open(path.clone())?)?;
    world.components.clear();
    for (_, v) in scene.entities.iter() {
      let id = world.spawn_empty().id;
      for (t, d) in v {
        if let Some(loader) = COMPONENT_LOADERS.iter().find(|l| l.id.id() == *t) {
          trace!("Loading '{}' on {}.", loader.id.name, id);
          world.components.push_or_insert(
            loader.id,
            (
              id,
              (loader.load)(d.clone(), world.get_resource::<Assets>().unwrap()),
            ),
          )
        }
      }
    }
    info!("Loaded scene from '{}'.", path.display());
    Ok(())
  }
}
