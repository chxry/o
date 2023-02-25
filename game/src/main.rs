use phosphor::{Engine, Result};
use phosphor::ecs::{World, stage};
use phosphor_3d::{Transform, Camera, Model, Material, SkySettings, scenerenderer};
use phosphor_imgui::{
  imgui::{Ui, Drag},
  uirenderer,
};
use phosphor::log::LevelFilter;
use phosphor::math::Vec3;
use phosphor::assets::Assets;

fn main() -> Result {
  ezlogger::init(LevelFilter::Debug)?;
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
    .spawn("plane")
    .insert(Transform::new().scale(Vec3::splat(7.5)))
    .insert(Model::new(assets.load("plane.obj")?))
    .insert(Material::Color(Vec3::splat(0.75)));
  world
    .spawn("garf")
    .insert(Transform::new().rot(Vec3::new(0.0, 90.0, 0.0)))
    .insert(Model::new(assets.load("garfield.obj")?))
    .insert(Material::Texture(assets.load("garfield.png")?));
  world
    .spawn("cylinder")
    .insert(Transform::new().pos(Vec3::new(5.0, 2.0, 0.0)))
    .insert(Model::new(assets.load("cylinder.obj")?))
    .insert(Material::Texture(assets.load("brick.jpg")?));
  phosphor::scene::Scene::save(world, "test.scene".into())?;
  Ok(())
}

fn draw(world: &mut World) -> Result {
  let teapot = world.get_name("garf").unwrap();
  let ui = world.get_resource::<Ui>().unwrap();
  ui.window("debug").always_auto_resize(true).build(|| {
    Drag::new("rotation")
      .speed(0.5)
      .build_array(&ui, teapot.get::<Transform>().unwrap().rotation.as_mut());
  });
  world.get_resource::<SkySettings>().unwrap().dir.x += 1.0;
  Ok(())
}
