use phosphor::ecs::World;
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

fn render(_: &mut World, _: &Ui) {}
