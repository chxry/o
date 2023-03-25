use std::ffi::CString;
use libfmod::{System, Sound as FmodSound, Channel};
use libfmod::ffi::{
  FMOD_INIT_3D_RIGHTHANDED, FMOD_3D, FMOD_VECTOR, FMOD_System_GetDriverInfo,
  FMOD_System_Set3DListenerAttributes, FMOD_Channel_Set3DAttributes,
};
use phosphor::assets::Handle;
use phosphor::ecs::{World, stage};
use phosphor::{Result, asset, component};
use phosphor::log::debug;
use phosphor::math::Vec3;
use phosphor_3d::{Camera, Transform};
use serde::{Serialize, Deserialize};

pub use libfmod as fmod;

pub struct FmodOptions {
  pub play_on_start: bool,
}

impl FmodOptions {
  const DEFAULT: Self = Self {
    play_on_start: true,
  };
}

pub struct FmodContext {
  pub system: System,
  pub ver: String,
}

pub fn fmod_plugin(world: &mut World) -> Result {
  let system = System::create().unwrap();
  system.init(512, FMOD_INIT_3D_RIGHTHANDED, None).unwrap();
  let mut ver = format!("{:x}", system.get_version().unwrap());
  ver.insert(1, '.');
  ver.insert(4, '.');
  unsafe {
    let name = CString::from_vec_unchecked(vec![0; 64]);
    FMOD_System_GetDriverInfo(
      system.as_mut_ptr(),
      0,
      name.as_ptr() as _,
      64,
      0 as _,
      0 as _,
      0 as _,
      0 as _,
    );
    debug!("Created FMOD {} system on '{}'. ", ver, name.to_str()?);
    world.add_resource(FmodContext { system, ver });
  }

  let options = match world.get_resource::<FmodOptions>() {
    Some(o) => o,
    None => &FmodOptions::DEFAULT,
  };
  if options.play_on_start {
    world.add_system(stage::START, fmod_start);
  }
  world.add_system(stage::PRE_DRAW, fmod_predraw);
  Ok(())
}

fn fmod_start(world: &mut World) -> Result {
  for (_, a) in world.query::<AudioSource>() {
    if a.play_on_start {
      a.play(world);
    }
  }
  Ok(())
}

fn fmod_predraw(world: &mut World) -> Result {
  let fmod = world.get_resource::<FmodContext>().unwrap();
  if let Some((e, _)) = world.query::<Camera>().get(0) {
    if let Some(cam_t) = e.get_one::<Transform>() {
      unsafe {
        FMOD_System_Set3DListenerAttributes(
          fmod.system.as_mut_ptr(),
          0,
          &fvec(cam_t.position),
          &fvec(Vec3::ZERO),
          &fvec(cam_t.rotation * Vec3::NEG_Z),
          &fvec(cam_t.rotation * Vec3::Y),
        );
      }
    }
  }
  for (e, a) in world.query::<AudioSource>() {
    if let Some(channel) = a.channel {
      if let Some(t) = e.get_one::<Transform>() {
        unsafe {
          FMOD_Channel_Set3DAttributes(channel.as_mut_ptr(), &fvec(t.position), &fvec(Vec3::ZERO));
        }
      }
    }
  }
  fmod.system.update().unwrap();
  Ok(())
}

#[asset(load_sound)]
pub struct Sound(pub FmodSound);

fn load_sound(world: &mut World, path: &str) -> Result<Sound> {
  Ok(Sound(
    world
      .get_resource::<FmodContext>()
      .unwrap()
      .system
      .create_sound(path, FMOD_3D, None)
      .unwrap(),
  ))
}

#[derive(Serialize, Deserialize)]
#[component]
pub struct AudioSource {
  pub sound: Handle<Sound>,
  pub pitch: f32,
  pub play_on_start: bool,
  #[serde(skip)]
  channel: Option<Channel>,
}

impl AudioSource {
  pub fn new(sound: Handle<Sound>) -> Self {
    Self {
      sound,
      pitch: 1.0,
      play_on_start: true,
      channel: None,
    }
  }

  pub fn play(&mut self, world: &World) {
    let channel = world
      .get_resource::<FmodContext>()
      .unwrap()
      .system
      .play_sound(self.sound.0, None, false)
      .unwrap();
    channel.set_pitch(self.pitch).unwrap();
    self.channel = Some(channel);
  }
}

fn fvec(v: Vec3) -> FMOD_VECTOR {
  FMOD_VECTOR {
    x: v.x,
    y: v.y,
    z: v.z,
  }
}
