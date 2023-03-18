use phosphor::ecs::World;
use phosphor_3d::SkySettings;
use phosphor_imgui::imgui::{Ui, WindowFlags, Drag};
use crate::panels::Panel;

pub fn init() -> Panel {
  Panel {
    title: "\u{f765} Environment",
    flags: WindowFlags::empty(),
    vars: &[],
    open: true,
    render,
  }
}

fn render(world: &mut World, ui: &Ui) {
  let sky = world.get_resource::<SkySettings>().unwrap();
  Drag::new("light dir")
    .speed(0.5)
    .build_array(&ui, sky.dir.as_mut());
}
