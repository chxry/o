use phosphor::ecs::World;
use phosphor_ui::imgui::{Ui, WindowFlags, StyleVar};
use crate::panels::Panel;

pub fn init() -> Panel {
  Panel {
    title: "\u{e88e} Info",
    flags: WindowFlags::NO_RESIZE | WindowFlags::ALWAYS_AUTO_RESIZE,
    vars: &[StyleVar::WindowPadding([20.0, 20.0])],
    open: false,
    render: &render,
  }
}

fn render(_: &mut World, ui: &Ui) {
  ui.set_next_item_width(320.0);
  let font = ui.push_font(ui.fonts().fonts()[1]);
  ui.text("Phosphor");
  font.pop();
  ui.text("github.com/chxry/phosphor");
  ui.same_line_with_pos(230.0);
  ui.text(env!("CARGO_PKG_VERSION"));
  ui.separator();
  ui.text("a");
}
