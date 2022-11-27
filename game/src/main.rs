use phosphor::{Engine, Result};
use phosphor::ecs::{World, Stage};
use phosphor::gfx::Mesh;
use phosphor_3d::{Transform, Camera, Material, scenerenderer};
use phosphor_ui::{
  imgui::{Ui, Drag},
  uirenderer,
};
use phosphor::log::LevelFilter;
use phosphor::math::Vec3;
use phosphor::assets::Assets;

fn main() -> Result {
  env_logger::builder().filter_level(LevelFilter::Info).init();
  Engine::new()
    .add_system(Stage::Start, &scenerenderer)
    .add_system(Stage::Start, &uirenderer)
    .add_system(Stage::Start, &start)
    .add_system(Stage::Draw, &draw)
    .run()
}

fn start(world: &mut World) -> Result {
  let assets = world.get_resource::<Assets>().unwrap();
  world
    .spawn("cam")
    .insert(
      Transform::new()
        .pos(Vec3::new(0.0, 1.0, -10.0))
        .rot(Vec3::new(0.0, 90.0, 0.0)),
    )
    .insert(Camera::new(80.0, [0.1, 100.0]));
  world
    .spawn("teapot")
    .insert(Transform::new())
    .insert(assets.load::<Mesh>("res/teapot.obj")?)
    .insert(Material::texture(assets.load("res/brick.jpg")?));
  Ok(())
}

fn draw(world: &mut World) -> Result {
  let teapot = world.get_name("teapot").unwrap();
  let ui = world.get_resource::<Ui>().unwrap();
  ui.window("debug").always_auto_resize(true).build(|| {
    Drag::new("rotation")
      .speed(0.5)
      .build_array(&ui, teapot.get::<Transform>().unwrap().rotation.as_mut());
  });
  Ok(())
}
