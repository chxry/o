use std::any::Any;
use phosphor::TypeIdNamed;
use phosphor::ecs::World;
use phosphor::assets::{Assets, Handle};
use phosphor_ui::imgui::{Ui, WindowFlags};
use crate::panels::Panel;

pub struct SelectedAsset(Option<(TypeIdNamed, Handle<dyn Any>)>);

pub fn init(world: &mut World) -> Panel {
  world.add_resource(SelectedAsset(None));
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
  let selected = world.get_resource::<SelectedAsset>().unwrap();
  for (t, v) in assets.handles.iter() {
    let mut pos = ui.cursor_pos();
    for handle in v.0.iter() {
      let id = ui.push_id(handle.name.clone());
      if ui
        .selectable_config("##")
        .size([80.0, 80.0])
        .selected(
          selected
            .0
            .as_ref()
            .map(|h| h.1.name == handle.name)
            .unwrap_or(false),
        )
        .build()
      {
        *selected = SelectedAsset(Some((t.clone(), handle.clone())));
      }
      if let Some(_) = ui.drag_drop_source_config("drag").begin() {
        ui.text(handle.name.clone());
      }
      ui.set_cursor_pos([pos[0] + 24.0, pos[1] + 6.0]);
      let font = ui.push_font(ui.fonts().fonts()[1]);
      ui.text("\u{f15b}");
      font.pop();
      ui.set_cursor_pos([pos[0] + 6.0, pos[1] + 56.0]);
      ui.text(handle.name.clone().split("/").last().unwrap());
      pos[0] += 88.0;
      ui.set_cursor_pos(pos);
      id.pop();
    }
  }
  let [w, _] = ui.window_size();
  ui.set_cursor_pos([w - 320.0, 24.0]);
  ui.child_window("##")
    .border(true)
    .build(|| match &selected.0 {
      Some(h) => {
        let font = ui.push_font(ui.fonts().fonts()[1]);
        ui.text("\u{f15b}");
        font.pop();
        ui.same_line();
        let pos = ui.cursor_pos();
        ui.text(h.1.name.clone());
        ui.set_cursor_pos([pos[0], pos[1] + 16.0]);
        ui.text_disabled(h.0.name.clone());
      }
      None => ui.text("\u{f071} No asset selected."),
    });
}
