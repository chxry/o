use phosphor::ecs::World;
use phosphor_imgui::imgui::{Ui, WindowFlags};
use phosphor::log::Level;
use crate::panels::Panel;

struct LogLevel(Level);

pub fn init(world: &mut World) -> Panel {
  world.add_resource(LogLevel(Level::Debug));
  Panel {
    title: "\u{f4a6} Log",
    flags: WindowFlags::empty(),
    vars: &[],
    open: true,
    render: &render,
  }
}

fn render(world: &mut World, ui: &Ui) {
  let id = ui.push_id("shit");
  let level = world.get_resource::<LogLevel>().unwrap();
  let records = ezlogger::records()
    .iter()
    .filter(|r| level.0 >= r.level)
    .collect::<Vec<_>>();
  ui.set_next_item_width(128.0);
  if let Some(_) = ui.begin_combo("Filter", level.0.as_str()) {
    for l in Level::iter() {
      if ui.selectable(l.as_str()) {
        *level = LogLevel(l);
      }
    }
  }
  ui.same_line();
  ui.text(format!("| {} messages", records.len()));
  for record in records {
    let font = ui.push_font(ui.fonts().fonts()[1]);
    ui.set_window_font_scale(0.65);
    let (color, icon) = match record.level {
      Level::Error => ([0.749, 0.38, 0.416, 1.0], "\u{f06a}"),
      Level::Warn => ([0.922, 0.796, 0.545, 1.0], "\u{f06a}"),
      Level::Info => ([0.639, 0.745, 0.549, 1.0], "\u{f05a}"),
      Level::Debug => ([0.506, 0.631, 0.757, 1.0], "\u{f059}"),
      Level::Trace => ([0.4, 0.4, 0.4, 1.0], "\u{f059}"),
    };
    ui.text_colored(color, icon);
    ui.set_window_font_scale(1.0);
    font.pop();
    if ui.is_item_hovered() {
      ui.tooltip_text(record.level.as_str());
    }
    ui.same_line();
    let pos = ui.cursor_pos();
    ui.text(record.msg.clone());
    ui.set_cursor_pos([pos[0], pos[1] + 16.0]);
    let (h, m, s) = record.time.to_hms();
    ui.text_disabled(format!(
      "{:02}:{:02}:{:02} | {}",
      h,
      m,
      s,
      record.module.clone()
    ));
    ui.separator();
  }
  id.pop();
}
