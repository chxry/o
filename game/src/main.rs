use phosphor::{Engine, Result};
use phosphor::ecs::{World, stage};
use phosphor_3d::{Transform, Camera, Model, Material, SkySettings, scenerenderer_plugin};
use phosphor_imgui::{
  imgui::{Ui, Drag},
  imgui_plugin,
};
use phosphor_fmod::{AudioSource, fmod_plugin};
use phosphor::log::LevelFilter;
use phosphor::math::Vec3;
use phosphor::assets::Assets;
use phosphor::scene::Scene;

fn main() -> Result {
  ezlogger::init(LevelFilter::Debug)?;
  Engine::new()
    .add_system(stage::INIT, scenerenderer_plugin)
    .add_system(stage::INIT, imgui_plugin)
    .add_system(stage::INIT, fmod_plugin)
    .add_system(stage::INIT, start)
    .add_system(stage::DRAW, draw)
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
    .insert(Material::Texture(assets.load("garfield.png")?))
    .insert(AudioSource::new(assets.load("portal-radio.mp3")?));
  world
    .spawn("cylinder")
    .insert(Transform::new().pos(Vec3::new(5.0, 2.0, 0.0)))
    .insert(Model::new(assets.load("cylinder.obj")?))
    .insert(Material::Texture(assets.load("brick.jpg")?));
  Scene::save(world, "test.scene".into())?;
  Ok(())
}

fn draw(world: &mut World) -> Result {
  let garf = world.get_name("garf").unwrap();
  let ui = world.get_resource::<Ui>().unwrap();
  ui.window("debug").always_auto_resize(true).build(|| {
    Drag::new("rotation")
      .speed(0.5)
      .build_array(&ui, garf.get_one::<Transform>().unwrap().rotation.as_mut());
  });
  world.get_resource::<SkySettings>().unwrap().dir.x += 1.0;
  Ok(())
}
