# ECS Introduction

Phosphor is built around an Entity Component System, which is often shortened to ECS. This means our project's logic and data is broken up into the following groups:
- **Entities:** Entities are an identifier for things in our game. They contain no data except a unique identifier.
- **Components:** Components are Rust structs that are attached to our entities. Any Rust type can be a component, for example `Transform`, which is used to represent the location of entities in a scene.
- **Systems:** Systems are functions that are run at specific stages of our game. For example the following system prints the position of entities in a scene.
```rs
fn print_positions(world: &mut World) -> Result<()> {
  for (_, transform) in world.query::<Transform>() {
    info!("{}", transform.position);
  }
  Ok(())
}
```
- **Resources:** Resources are used to represent global data. They can be any Rust type, for example `Renderer`, which holds OpenGL and window state.

## Creating a system

A system can be any function with the signature `fn sys(world: &mut World) -> Result<()>`.

Lets create our first system and run it on startup. First import `World` and `Stage` from `phosphor::ecs`, and also `info` from `phosphor::log`.

```rs
use phosphor::ecs::{World, Stage};
use phosphor::log::{LevelFilter, info};
```

Now we can define our system and add run it on `Stage::Start`.

```rs
fn main() -> Result<()> {
  ...
  Engine::new()
    .add_system(Stage::Start, &hello)
    .run()
}

fn hello(world: &mut World) -> Result<()> {
  info!("Hello Phosphor!");
  Ok(())
}
```

After running our game we should see the following in our terminal:

```
[2022-11-03T21:30:13Z INFO  game] Hello Phosphor!
```

## Creating components

Lets add an entity with a `Health(u8)` component. We can create our entity using `world.spawn()` and use `entity.insert()` to attach any type as a component.

```rs
struct Health(u8);

fn hello(world: &mut World) -> Result<()> {
  world.spawn().insert(Health(100));
  Ok(())
}
```

## Accessing components

After running our game, we won't see anything, so lets create a system to check the health of our entity, using `world.query()` to get all the components of a type.

```rs
fn main() -> Result<()> {
  ...
  Engine::new()
    .add_system(Stage::Start, &hello)
    .add_system(Stage::Draw, &health_check)
    .run()
}


fn health_check(world: &mut World) -> Result<()> {
  let (entity, health) = &world.query::<Health>()[0];
  info!("Entity {} has a health of {}.", entity.id, health.0);
  Ok(())
}
```

After runnning our game we should see the following repeated in our terminal:

```
[2022-11-03T21:41:31Z INFO  game] Entity 1 has a health of 100.
```
