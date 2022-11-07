mod panels;

use phosphor::{Engine, Result, mutate};
use phosphor::ecs::{Stage, World};
use phosphor::log::LevelFilter;
use phosphor_ui::{uirenderer, UiRendererOptions};
use phosphor_ui::imgui::Ui;
use phosphor_3d::{scenerenderer, SceneRendererOptions};
use crate::panels::{Panel, setup_panels, update_panels};

fn main() -> Result<()> {
  env_logger::builder().filter_level(LevelFilter::Info).init();
  Engine::new()
    .add_resource(SceneRendererOptions { draw_stage: false })
    .add_resource(UiRendererOptions {
      docking: true,
      ini_path: Some("phosphor_editor/ui.ini"),
    })
    .add_system(Stage::Start, &uirenderer)
    .add_system(Stage::Start, &scenerenderer)
    .add_system(Stage::Start, &setup_panels)
    .add_system(Stage::PreDraw, &update_panels)
    .add_system(Stage::Draw, &draw_ui)
    .run()
}

fn draw_ui(world: &mut World) -> Result<()> {
  let ui = world.get_resource::<Ui>().unwrap();
  let panels = world.get_resource::<Vec<Box<dyn Panel>>>().unwrap();
  ui.main_menu_bar(|| {
    ui.menu("View", || {
      for panel in panels.iter_mut() {
        ui.menu_item_config(panel.title())
          .build_with_ref(panel.open());
      }
    });
  });
  for panel in panels {
    if *panel.open() {
      ui.window(panel.title()).build(|| {
        panel.render(mutate(world), ui);
      });
    }
  }
  Ok(())
}
