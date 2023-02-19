use phosphor::ecs::{World, Name};
use phosphor_imgui::imgui::{Ui, WindowFlags};
use crate::SelectedEntity;
use crate::panels::Panel;

pub fn init() -> Panel {
  Panel {
    title: "\u{e1e0} Outline",
    flags: WindowFlags::empty(),
    vars: &[],
    open: true,
    render: &render,
  }
}

fn render(world: &mut World, ui: &Ui) {
  let [w, _] = ui.window_size();
  let selected = world.get_resource::<SelectedEntity>().unwrap();
  for (e, n) in world.query::<Name>() {
    let id = ui.push_id_usize(e.id);
    if ui
      .selectable_config(n.0.clone())
      .selected(e.id == selected.0.unwrap_or_default())
      .build()
    {
      *selected = SelectedEntity(Some(e.id));
    }
    id.end();
  }
  ui.separator();
  if ui.button_with_size("\u{2b} Add Entity", [w, 0.0]) {
    world.spawn("New");
  }
}
