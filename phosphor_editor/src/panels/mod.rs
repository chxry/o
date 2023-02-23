mod scene;
mod outline;
mod environment;
mod inspector;
mod settings;
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

// use linkme for this
pub fn setup_panels(world: &mut World) -> Result<()> {
  let scene = scene::init(world)?;
  let outline = outline::init();
  let environment = environment::init();
  let inspector = inspector::init(world);
  let settings = settings::init(world);
  let log = log::init(world);
  let assets = assets::init(world);
  world.add_resource(vec![
    scene,
    outline,
    environment,
    inspector,
    settings,
    log,
    assets,
  ]);
  Ok(())
}
