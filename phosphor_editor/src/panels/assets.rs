use phosphor::ecs::World;
use phosphor::assets::Assets;
use phosphor_ui::imgui::{Ui, WindowFlags};
use crate::panels::Panel;

pub fn init() -> Panel {
  Panel {
    title: "\u{f660} Assets",
    flags: WindowFlags::empty(),
    vars: &[],
    open: true,
    render: &render,
  }
}

fn render(world: &mut World, ui: &Ui) {
  let assets = world.get_resource::<Assets>().unwrap();
  for (t, v) in assets.handles.iter() {
    ui.text(t.name);
    for a in v {
      ui.same_line();
      ui.text(a.name.clone());
    }
  }
}
