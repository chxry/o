use engine::Engine;
use engine::ecs::{Stage, World};
use engine::log::{LevelFilter, info};
use anyhow::Result;

fn main() -> Result<()> {
  env_logger::builder().filter_level(LevelFilter::Info).init();
  Engine::new().add_system(Stage::Start, &hello_system).run()
}

fn hello_system(world: &mut World) {
  info!("hello");
  let test = world.spawn();
  world.insert(&test, 5);
  for (e, i) in world.query::<i32>() {
    info!("{:?} {}", e, i);
  }
}
