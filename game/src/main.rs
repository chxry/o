use phosphor::{Engine, Result, DeltaTime};
use phosphor::ecs::{World, stage};
use phosphor_3d::{Transform, Camera, Model, Material, Light, scenerenderer_plugin};
use phosphor_fmod::{AudioSource, fmod_plugin};
use phosphor::log::LevelFilter;
use phosphor::math::Vec3;
use phosphor::assets::Assets;
use phosphor::scene::Scene;
use phosphor::gfx::Renderer;
use phosphor::glfw::{CursorMode, Key, Action};
use dolly::rig::CameraRig;
use dolly::handedness::RightHanded;
use dolly::drivers::{Position, YawPitch, Smooth};

struct LastPos(f32, f32);

fn main() -> Result {
  ezlogger::init(LevelFilter::Debug)?;
  Engine::new()
    .add_resource(LastPos(0.0, 0.0))
    .add_system(stage::INIT, scenerenderer_plugin)
    .add_system(stage::INIT, fmod_plugin)
    .add_system(stage::INIT, start)
    .add_system(stage::DRAW, camera)
    .run()
}

fn start(world: &mut World) -> Result {
  let renderer = world.get_resource::<Renderer>().unwrap();
  renderer.window.set_cursor_mode(CursorMode::Disabled);
  let assets = world.get_resource::<Assets>().unwrap();
  world
    .spawn("cam")
    .insert(Transform::new())
    .insert(Camera::new(80.0, [0.1, 100.0]))
    .insert(
      CameraRig::<RightHanded>::builder()
        .with(Position::new(Vec3::new(0.0, 1.0, -10.0)))
        .with(YawPitch::new())
        .with(Smooth::new_position_rotation(1.0, 0.5))
        .build(),
    );
  world
    .spawn("plane")
    .insert(Transform::new().scale(Vec3::splat(10.0)))
    .insert(Model::new(assets.load("plane.obj")?))
    .insert(Material::Color {
      color: Vec3::splat(0.75),
      spec: 0.5,
    });
  world
    .spawn("garf")
    .insert(Transform::new().rot_euler(90.0, 0.0, 0.0))
    .insert(Model::new(assets.load("garfield.obj")?))
    .insert(Material::Texture {
      tex: assets.load("garfield.png")?,
      spec: 0.5,
    })
    .insert(AudioSource::new(assets.load("portal-radio.mp3")?));
  world
    .spawn("cylinder")
    .insert(Transform::new().pos(Vec3::new(5.0, 2.0, 0.0)))
    .insert(Model::new(assets.load("cylinder.obj")?))
    .insert(Material::Texture {
      tex: assets.load("brick.jpg")?,
      spec: 0.5,
    });
  insert_light(world, "red", Vec3::X, (2.0, -2.0))?;
  insert_light(world, "green", Vec3::Y, (0.0, -2.0))?;
  insert_light(world, "blue", Vec3::Z, (1.0, -4.0))?;
  Scene::save(world, "test.scene".into())?;
  Ok(())
}

fn insert_light(world: &mut World, name: &str, col: Vec3, pos: (f32, f32)) -> Result {
  world
    .spawn(name)
    .insert(
      Transform::new()
        .pos(Vec3::new(pos.0, 1.5, pos.1))
        .scale(Vec3::splat(0.1)),
    )
    .insert(Light::new(col).strength(1.25))
    .insert(Model::new(
      world.get_resource::<Assets>().unwrap().load("sphere.obj")?,
    ));
  Ok(())
}

fn camera(world: &mut World) -> Result {
  let renderer = world.get_resource::<Renderer>().unwrap();
  let cam = world.get_name("cam").unwrap();
  let cam_t = cam.get_one::<Transform>().unwrap();

  let rig = cam.get_one::<CameraRig>().unwrap();
  let t = rig.update(world.get_resource::<DeltaTime>().unwrap().0);
  cam_t.position = t.position;
  cam_t.rotation = t.rotation;

  let r = rig.driver_mut::<YawPitch>();
  let last_pos = world.get_resource::<LastPos>().unwrap();
  let pos = renderer.window.get_cursor_pos();
  let pos = (pos.0 as f32, pos.1 as f32);
  r.yaw_degrees -= (pos.0 - last_pos.0) * 0.2;
  r.pitch_degrees -= (pos.1 - last_pos.1) * 0.2;
  *last_pos = LastPos(pos.0, pos.1);

  let pos = rig.driver_mut::<Position>();
  if renderer.window.get_key(Key::W) == Action::Press {
    pos.translate(t.forward() * 0.2);
  }
  if renderer.window.get_key(Key::A) == Action::Press {
    pos.translate(t.right() * -0.2);
  }
  if renderer.window.get_key(Key::S) == Action::Press {
    pos.translate(t.forward() * -0.2);
  }
  if renderer.window.get_key(Key::D) == Action::Press {
    pos.translate(t.right() * 0.2);
  }
  Ok(())
}
