use engine::Engine;
use engine::ecs::{Stage, Context};
use engine::scene::{Transform, Camera, Material, scenerenderer};
use engine::gfx::{Mesh, Texture};
use engine::ui::{uirenderer, ui_frame};
use engine::log::LevelFilter;
use engine::math::Vec3;
use anyhow::Result;

fn main() -> Result<()> {
  env_logger::builder().filter_level(LevelFilter::Info).init();
  Engine::new()
    .add_system(Stage::Start, &scenerenderer)
    .add_system(Stage::Start, &uirenderer)
    .add_system(Stage::Start, &start)
    .add_system(Stage::Draw, &draw)
    .run()
}

fn start(ctx: Context) -> Result<()> {
  let cam = ctx.world.spawn();
  ctx.world.insert(
    &cam,
    Transform::new()
      .pos(Vec3::new(0.0, 1.0, -10.0))
      .rot_euler(Vec3::new(0.0, 0.0, 1.5)),
  );
  ctx.world.insert(&cam, Camera::new(0.8));
  let teapot = ctx.world.spawn();
  ctx.world.insert(&teapot, Transform::new());
  ctx
    .world
    .insert(&teapot, Mesh::load(ctx.renderer, "res/teapot.obj")?);
  ctx.world.insert(
    &teapot,
    Material::Textured(Texture::load(ctx.renderer, "res/floppa.jpg")?),
  );
  Ok(())
}

fn draw(ctx: Context) -> Result<()> {
  ui_frame(ctx, |ui| {
    ui.show_demo_window(&mut true);
  })?;
  Ok(())
}
