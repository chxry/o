use engine::Engine;
use engine::ecs::{Stage, Context, Transform};
use engine::gfx::Mesh;
use engine::log::LevelFilter;
use anyhow::Result;

fn main() -> Result<()> {
  env_logger::builder().filter_level(LevelFilter::Info).init();
  Engine::new().add_system(Stage::Start, &start).run()
}

fn start(ctx: Context) -> Result<()> {
  let teapot = ctx.world.spawn();
  ctx.world.insert(&teapot, Transform::new());
  ctx
    .world
    .insert(&teapot, Mesh::load(ctx.renderer, "res/teapot.obj")?);
  Ok(())
}
