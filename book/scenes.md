# Scenes

Our game is looking quite empty right now, so lets render a basic scene. Phosphor is designed to be modular, so we have to enable the scene renderer first.

```rs
use phosphor::scene::{scenerenderer};

fn main() -> Result<()> {
  ...
  Engine::new()
    .add_system(Stage::Start, &scenerenderer)
    .run()
}
```

We won't see a difference since we haven't added any models, but lets download a [model](https://raw.githubusercontent.com/chxry/o/master/res/teapot.obj).

Now we can use a start system to add a `Camera` and `Mesh` to the scene. Both entities will also need a `Transform` to represent their position. We will need to use the `Renderer` resource to create our mesh. 
```rs
use phosphor::scene::{Camera, Transform, scenerenderer};
use phosphor::gfx::{Renderer, Mesh};
use phosphor::math::Vec3;

fn main() -> Result<()> {
  ...
  Engine::new()
    .add_system(Stage::Start, &scenerenderer)
    .add_system(Stage::Start, &setup)
    .run()
}

fn setup(world: &mut World) -> Result<()> {
  let renderer = world.get_resource::<Renderer>().unwrap();
  world
    .spawn()
    .insert(
      Transform::new()
        .pos(Vec3::new(0.0, 1.0, -10.0))
        .rot_euler(Vec3::new(0.0, 0.0, 1.5)),
    )
    .insert(Camera::new(0.8, 0.1..100.0));
  world
    .spawn()
    .insert(Transform::new())
    .insert(Mesh::load(renderer, "teapot.obj")?);
  Ok(())
}
```

After running our game we should now see a blank teapot.

## Textures

Lets add a texture to our teapot, first download a [texture](https://raw.githubusercontent.com/chxry/o/master/res/brick.jpg).

Now we can add a `Material` to our `Mesh`.

```rs
use phosphor::scene::{Camera, Transform, Material, scenerenderer};
use phosphor::gfx::{Renderer, Mesh, Texture};

...
world
  .spawn()
  .insert(Transform::new())
  .insert(Mesh::load(renderer, "teapot.obj")?)
  .insert(Material::Textured(Texture::load(
    renderer,
    "brick.jpg",
  )?));
...
```

Our teapot should now have a brick texture in game.
