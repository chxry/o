use phosphor::ecs::World;
use phosphor_3d::LightDir;
use phosphor_imgui::imgui::{Ui, WindowFlags, Drag};
use crate::panels::Panel;

pub fn init() -> Panel {
  Panel {
    title: "\u{f765} Environment",
    flags: WindowFlags::empty(),
    vars: &[],
    open: true,
    render: &render,
  }
}

fn render(world: &mut World, ui: &Ui) {
  let light_dir = world.get_resource::<LightDir>().unwrap().0.as_mut();
  Drag::new("light dir")
    .speed(0.5)
    .display_format("%g")
    .build_array(&ui, light_dir.as_mut());
}
