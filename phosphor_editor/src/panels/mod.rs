mod scene;
mod outline;
mod inspector;
mod info;
mod log;
mod assets;

use phosphor::Result;
use phosphor::ecs::World;
use phosphor_imgui::imgui::{Ui, WindowFlags, StyleVar};

pub struct Panel {
  pub title: &'static str,
  pub flags: WindowFlags,
  pub vars: &'static [StyleVar],
  pub open: bool,
  pub render: &'static dyn Fn(&mut World, &Ui),
}

pub fn setup_panels(world: &mut World) -> Result<()> {
  let scene = scene::init(world)?;
  let outline = outline::init();
  let inspector = inspector::init(world);
  let info = info::init();
  let log = log::init(world);
  let assets = assets::init(world);
  world.add_resource(vec![scene, outline, inspector, info, log, assets]);
  Ok(())
}
