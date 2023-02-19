use phosphor::{Engine, Result};
use phosphor::ecs::{World, stage};
use phosphor::gfx::Mesh;
use phosphor_3d::{Transform, Camera, Model, Material, scenerenderer};
use phosphor_imgui::{
  imgui::{Ui, Drag},
  uirenderer,
};
use phosphor::log::LevelFilter;
use phosphor::math::Vec3;
use phosphor::assets::Assets;

fn main() -> Result {
  shitlog::init(LevelFilter::Trace)?;
  Engine::new()
    .add_system(stage::START, &scenerenderer)
    .add_system(stage::START, &uirenderer)
    .add_system(stage::START, &start)
    .add_system(stage::DRAW, &draw)
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
    .insert(Transform::new().rot(Vec3::new(0.0, 90.0, 0.0)))
    .insert(Model(assets.load::<Mesh>("res/teapot.obj")?))
    .insert(Material::Texture(assets.load("res/brick.jpg")?));
  phosphor::scene::Scene::save(world, "test.scene")?;
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
