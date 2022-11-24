use phosphor::ecs::{World, Name};
use phosphor_ui::imgui::{Ui, WindowFlags};
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
    if ui
      .selectable_config(n.0.clone())
      .selected(e.id == selected.0.unwrap_or_default())
      .build()
    {
      *selected = SelectedEntity(Some(e.id));
    }
  }
  ui.separator();
  ui.button_with_size("\u{2b} Add Entity", [w, 0.0]);
}
