use phosphor::{Result, Engine};
use phosphor::ecs::{World, Stage};
use phosphor_3d::{Transform, Camera, Material, scenerenderer};
use phosphor::gfx::{Mesh, Texture};
use phosphor_ui::{
  uirenderer,
  imgui::{Ui, Drag},
};
use phosphor::log::LevelFilter;
use phosphor::math::Vec3;

fn main() -> Result<()> {
  env_logger::builder().filter_level(LevelFilter::Info).init();
  Engine::new()
    .add_system(Stage::Start, &scenerenderer)
    .add_system(Stage::Start, &uirenderer)
    .add_system(Stage::Start, &start)
    .add_system(Stage::Draw, &draw)
    .run()
}

fn start(world: &mut World) -> Result<()> {
  world
    .spawn("cam")
    .insert(
      Transform::new()
        .pos(Vec3::new(0.0, 1.0, -10.0))
        .rot_euler(Vec3::new(0.0, 0.0, 1.5)),
    )
    .insert(Camera::new(0.8, 0.1..100.0));
  world
    .spawn("teapot")
    .insert(Transform::new())
    .insert(Mesh::load("res/teapot.obj")?)
    .insert(Material::Textured(Texture::load("res/brick.jpg")?));
  Ok(())
}

fn draw(world: &mut World) -> Result<()> {
  let teapot = world.get_name("teapot").unwrap();
  let ui = world.get_resource::<Ui>().unwrap();
  ui.window("debug").always_auto_resize(true).build(|| {
    Drag::new("position")
      .speed(0.05)
      .build_array(&ui, teapot.get::<Transform>().unwrap().position.as_mut());
  });
  Ok(())
}
