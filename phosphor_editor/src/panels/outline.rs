use phosphor::ecs::{World, Name};
use phosphor_3d::Transform;
use phosphor_imgui::imgui::{Ui, WindowFlags};
use crate::SelectedEntity;
use crate::panels::Panel;

pub fn init() -> Panel {
  Panel {
    title: "\u{e1e0} Outline",
    flags: WindowFlags::empty(),
    vars: &[],
    open: true,
    render,
  }
}

fn render(world: &mut World, ui: &Ui) {
  let [w, _] = ui.window_size();
  let selected = world.get_resource::<SelectedEntity>().unwrap();
  for (e, n) in world.query::<Name>() {
    let id = ui.push_id_usize(e.id);
    if ui
      .selectable_config(n.0.clone())
      .selected(selected.0.is_some_and(|s| s.id == e.id))
      .build()
    {
      *selected = SelectedEntity(Some(e));
    }
    id.pop();
  }
  ui.separator();
  if ui.button_with_size("\u{2b} Add Entity", [w, 0.0]) {
    world.spawn("New").insert(Transform::new());
  }
}
