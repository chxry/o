mod panels;

use phosphor::{Engine, Result, mutate};
use phosphor::ecs::{Stage, World};
use phosphor::log::LevelFilter;
use phosphor_ui::{uirenderer, UiRendererOptions};
use phosphor_ui::imgui::{Ui, StyleStackToken};
use crate::panels::{Panel, setup_panels};

pub struct SelectedEntity(Option<usize>);

fn main() -> Result<()> {
  env_logger::builder().filter_level(LevelFilter::Info).init();
  Engine::new()
    .add_resource(UiRendererOptions {
      docking: true,
      ini_path: Some("phosphor_editor/ui.ini"),
      fonts: &[
        &[
          ("res/roboto.ttf", 16.0, None),
          ("res/materialicons.ttf", 13.0, Some(&[0xe000, 0x10fffd, 0])),
        ],
        &[("res/helvetica-bold.ttf", 36.0, None)],
      ],
    })
    .add_resource(SelectedEntity(None))
    .add_system(Stage::Start, &uirenderer)
    .add_system(Stage::Start, &setup_panels)
    .add_system(Stage::Draw, &draw_ui)
    .run()
}

fn draw_ui(world: &mut World) -> Result<()> {
  let ui = world.get_resource::<Ui>().unwrap();
  let panels = world.get_resource::<Vec<Panel>>().unwrap();
  ui.main_menu_bar(|| {
    ui.menu("File", || {});
    ui.menu("View", || {
      for panel in panels.iter_mut() {
        ui.menu_item_config(panel.title)
          .build_with_ref(&mut panel.open);
      }
    });
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
