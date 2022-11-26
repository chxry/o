use phosphor::ecs::World;
use phosphor::gfx::Renderer;
use phosphor_ui::imgui::{Ui, WindowFlags, StyleVar};
use crate::panels::Panel;

pub fn init() -> Panel {
  Panel {
    title: "\u{f05a} Info",
    flags: WindowFlags::NO_RESIZE | WindowFlags::ALWAYS_AUTO_RESIZE,
    vars: &[StyleVar::WindowPadding([20.0, 20.0])],
    open: false,
    render: &render,
  }
}

fn render(world: &mut World, ui: &Ui) {
  let font = ui.push_font(ui.fonts().fonts()[1]);
  ui.text("\u{f5d3} Phosphor");
  font.pop();
  ui.text("github.com/chxry/phosphor");
  let [w, _] = ui.window_size();
  ui.same_line_with_pos(w - 52.0);
  ui.text(env!("CARGO_PKG_VERSION"));
  ui.separator();
  if let Some(_) = ui.tree_node("System info") {
    let renderer = world.get_resource::<Renderer>().unwrap();
    item(ui, "opengl ver", renderer.version);
    item(ui, "gpu", renderer.renderer);
  }
  if let Some(_) = ui.tree_node("Performance") {
    let io = ui.io();
    item(ui, "fps", &format!("{:.1}", io.framerate));
    item(ui, "dt", &format!("{:.1}ms", io.delta_time * 1000.0));
  }
}

fn item(ui: &Ui, label: &str, text: &str) {
  ui.text(label);
  let [w, _] = ui.window_content_region_max();
  let [x, _] = ui.calc_text_size(text);
  ui.same_line_with_pos(w - x);
  ui.text_disabled(text);
}
