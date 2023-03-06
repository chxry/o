use phosphor::ecs::World;
use phosphor::gfx::Renderer;
use phosphor_imgui::imgui::{Context, Ui, WindowFlags, StyleVar, dear_imgui_version};
use phosphor_fmod::FmodContext;
use crate::panels::Panel;

#[derive(PartialEq, Eq)]
enum SettingsPane {
  Appearance,
  About,
}

impl SettingsPane {
  const ALL: [Self; 2] = [Self::Appearance, Self::About];

  fn name(&self) -> &str {
    match self {
      Self::Appearance => "\u{f53f} Appearance",
      Self::About => "\u{f05a} About",
    }
  }
}

pub fn init(world: &mut World) -> Panel {
  world.add_resource(SettingsPane::Appearance);
  Panel {
    title: "\u{f013} Settings",
    flags: WindowFlags::empty(),
    vars: &[StyleVar::WindowPadding([0.0, 0.0])],
    open: false,
    render,
  }
}

fn render(world: &mut World, ui: &Ui) {
  let pane = world.get_resource::<SettingsPane>().unwrap();
  let pad = ui.push_style_var(StyleVar::WindowPadding([8.0, 8.0]));
  let space = ui.push_style_var(StyleVar::ItemSpacing([8.0, 8.0]));
  ui.child_window("l")
    .size([150.0, 0.0])
    .always_use_window_padding(true)
    .build(|| {
      for p in SettingsPane::ALL {
        if ui.selectable_config(p.name()).selected(*pane == p).build() {
          *pane = p
        }
      }
    });
  pad.pop();
  space.pop();
  ui.same_line_with_spacing(0.0, 0.0);
  let pad = ui.push_style_var(StyleVar::WindowPadding([8.0, 8.0]));
  ui.child_window("r").border(true).build(|| match pane {
    SettingsPane::Appearance => unsafe {
      static mut THEME: usize = 0; // too bored for a resource
      if ui.combo_simple_string("Theme", &mut THEME, &["Dark", "Nord", "Light"]) {
        let style = world.get_resource::<Context>().unwrap().style_mut();
        match THEME {
          0 => phosphor_imgui::theme_dark(style),
          1 => phosphor_imgui::theme_nord(style),
          2 => {
            style.use_light_colors();
          }
          _ => {}
        }
      }
    },
    SettingsPane::About => {
      let font = ui.push_font(ui.fonts().fonts()[1]);
      ui.text("\u{f5d3} Phosphor");
      font.pop();
      ui.text("github.com/chxry/phosphor");
      let [w, _] = ui.window_size();
      ui.same_line_with_pos(w - 40.0);
      ui.text(env!("CARGO_PKG_VERSION"));
      ui.separator();
      let renderer = world.get_resource::<Renderer>().unwrap();
      item(ui, "opengl ver", renderer.version);
      item(ui, "glfw ver", &phosphor::glfw::get_version_string());
      item(ui, "imgui ver", dear_imgui_version());
      let fmod = world.get_resource::<FmodContext>().unwrap();
      item(ui, "fmod ver", &fmod.ver);
      item(ui, "gpu", renderer.renderer);
    }
  });
  pad.pop();
}

fn item(ui: &Ui, label: &str, text: &str) {
  ui.text(label);
  let [w, _] = ui.window_content_region_max();
  let [x, _] = ui.calc_text_size(text);
  ui.same_line_with_pos(w - x);
  ui.text_disabled(text);
}
