mod panels;

use phosphor::{Engine, Result, mutate};
use phosphor::ecs::{World, stage};
use phosphor::scene::Scene;
use phosphor::log::LevelFilter;
use phosphor_imgui::{uirenderer, UiRendererOptions};
use phosphor_imgui::imgui::{Ui, StyleStackToken};
use crate::panels::{Panel, setup_panels};

pub struct SelectedEntity(Option<usize>);

const TEXT: &str = concat!(
  "\u{f5d3} ",
  env!("CARGO_PKG_NAME"),
  " ",
  env!("CARGO_PKG_VERSION")
);

fn main() -> Result<()> {
  shitlog::init(LevelFilter::Trace)?;
  Engine::new()
    .add_resource(UiRendererOptions {
      docking: true,
      ini_path: Some("phosphor_editor/ui.ini"),
      fonts: &[
        &[
          ("res/roboto.ttf", 16.0, None),
          ("res/fontawesome.ttf", 14.0, Some(&[0xe005, 0xf8ff, 0])),
        ],
        &[
          ("res/shingo.otf", 48.0, None),
          ("res/fontawesome.ttf", 48.0, Some(&[0xe005, 0xf8ff, 0])),
        ],
      ],
    })
    .add_resource(SelectedEntity(None))
    .add_system(stage::START, &uirenderer)
    .add_system(stage::START, &setup_panels)
    .add_system(stage::DRAW, &draw_ui)
    .run()
}

fn draw_ui(world: &mut World) -> Result<()> {
  let ui = world.get_resource::<Ui>().unwrap();
  let panels = world.get_resource::<Vec<Panel>>().unwrap();
  ui.main_menu_bar(|| {
    ui.menu("File", || {
      if ui.menu_item("Save") {
        Scene::save(world, "test.scene").unwrap();
      }
      if ui.menu_item("Load") {
        Scene::load(mutate(world), "test.scene").unwrap();
      }
    });
    ui.menu("View", || {
      for panel in panels.iter_mut() {
        ui.menu_item_config(panel.title)
          .build_with_ref(&mut panel.open);
      }
    });
    let [w, _] = ui.window_size();
    let [tx, _] = ui.calc_text_size(TEXT);
    ui.same_line_with_pos(w - tx - 16.0);
    ui.text_disabled(TEXT);
  });
  for panel in panels {
    if panel.open {
      let tokens: Vec<StyleStackToken> = panel.vars.iter().map(|v| ui.push_style_var(*v)).collect();
      ui.window(panel.title)
        .flags(panel.flags)
        .opened(&mut panel.open)
        .build(|| {
          (panel.render)(mutate(world), ui);
        });
      for token in tokens {
        token.pop();
      }
    }
  }
  Ok(())
}
