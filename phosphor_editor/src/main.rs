use phosphor::{Engine, Result};
use phosphor::ecs::{Stage, World};
use phosphor::log::LevelFilter;
use phosphor_ui::uirenderer;
use phosphor_ui::imgui::{Ui, MenuItem};

struct Panels {
  metrics: bool,
}

fn main() -> Result<()> {
  env_logger::builder().filter_level(LevelFilter::Info).init();
  Engine::new()
    .add_resource(Panels { metrics: true })
    .add_system(Stage::Start, &uirenderer)
    .add_system(Stage::Draw, &draw_panels)
    .run()
}

fn draw_panels(world: &mut World) -> Result<()> {
  let panels = world.get_resource::<Panels>().unwrap();
  let ui = world.get_resource::<Ui>().unwrap();
  ui.main_menu_bar(|| {
    ui.menu("View", || {
      MenuItem::new("Metrics").build_with_ref(&ui, &mut panels.metrics);
    });
  });
  if panels.metrics {
    ui.show_metrics_window(&mut panels.metrics);
  }
  Ok(())
}
