#![allow(clippy::redundant_pattern_matching)]
use phosphor::{Engine, Result, DeltaTime};
use phosphor::ecs::{World, stage};
use phosphor::log::LevelFilter;
use phosphor::math::Vec3;
use phosphor::assets::Assets;
use phosphor::scene::Scene;
use phosphor::gfx::{Renderer, Mesh, Query};
use phosphor::glfw::{CursorMode, Key, MouseButton, Action};
use phosphor_3d::{
  Transform, Camera, Model, Material, Light, SkySettings, ScenePerf, Tonemap, scenerenderer_plugin,
};
use phosphor_imgui::imgui_plugin;
use phosphor_imgui::imgui::{Ui, Condition, Drag};
use phosphor_fmod::{AudioSource, fmod_plugin};
use phosphor_rapier::rapier3d::prelude::*;
use phosphor_rapier::{RigidBodyBuilder, ColliderBuilder, Gravity, rapier_plugin};
use puffin_imgui::ProfilerUi;
use dolly::rig::CameraRig;
use dolly::handedness::RightHanded;
use dolly::drivers::{Position, YawPitch, Smooth};
use rand::Rng;

struct LastPos(f32, f32);

fn main() -> Result {
  ezlogger::init(LevelFilter::Debug)?;
  Engine::new()
    .add_resource(ProfilerUi::default())
    .add_resource(DebugRenderPipeline::new(
      DebugRenderStyle::default(),
      DebugRenderMode::empty(),
    ))
    .add_system(stage::INIT, scenerenderer_plugin)
    .add_system(stage::INIT, fmod_plugin)
    .add_system(stage::INIT, imgui_plugin)
    .add_system(stage::INIT, rapier_plugin)
    .add_system(stage::INIT, phosphor_rapier::rapier_debug_plugin)
    .add_system(stage::INIT, start)
    .add_system(stage::DRAW, camera)
    .add_system(stage::DRAW, ui)
    .run()
}

fn start(world: &mut World) -> Result {
  let renderer = world.get_resource::<Renderer>().unwrap();
  renderer.window.set_cursor_mode(CursorMode::Disabled);
  let pos = renderer.window.get_cursor_pos();
  world.add_resource(LastPos(pos.0 as f32, pos.1 as f32));
  let assets = world.get_resource::<Assets>().unwrap();
  world
    .spawn("cam")
    .insert(Transform::new())
    .insert(Camera::new(80.0, [0.1, 100.0]))
    .insert(
      CameraRig::<RightHanded>::builder()
        .with(Position::new(Vec3::new(0.0, 1.0, -10.0)))
        .with(YawPitch::new().yaw_degrees(180.0))
        .with(Smooth::new_position_rotation(1.0, 0.5))
        .build(),
    );
  let garf_rb = RigidBodyBuilder::dynamic().build(world);
  let garf_mesh = assets.load::<Mesh>("garfield.obj")?;
  world
    .spawn("garf")
    .insert(Transform::new().pos(Vec3::new(0.0, 0.0, 2.0)))
    .insert(Model::new(garf_mesh.clone()))
    .insert(Material {
      color: Vec3::ONE,
      tex: Some(assets.load("garfield.png")?),
      spec: 0.5,
      metallic: 0.5,
    })
    .insert(AudioSource::new(assets.load("portal-radio.mp3")?))
    .insert(
      ColliderBuilder::trimesh(&garf_mesh)
        .attach_rb(garf_rb)
        .build(world),
    )
    .insert(garf_rb);
  world
    .spawn("floor")
    .insert(Transform::new().scale(Vec3::new(10.0, 0.01, 10.0)))
    .insert(Model::new(assets.load("cube.obj")?))
    .insert(Material {
      color: Vec3::splat(0.75),
      tex: None,
      spec: 0.5,
      metallic: 0.5,
    })
    .insert(ColliderBuilder::cuboid(10.0, 0.01, 10.0).build(world));
  world
    .spawn("light")
    .insert(
      Transform::new()
        .pos(Vec3::new(2.0, 1.5, -2.0))
        .scale(Vec3::splat(0.1)),
    )
    .insert(Model::new(assets.load("sphere.obj")?))
    .insert(Light::new(Vec3::new(1.0, 0.0, 1.0)));

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

  let last_pos = world.get_resource::<LastPos>().unwrap();
  let pos = renderer.window.get_cursor_pos();
  let pos = (pos.0 as f32, pos.1 as f32);
  if renderer.window.get_cursor_mode() == CursorMode::Disabled {
    let r = rig.driver_mut::<YawPitch>();
    r.yaw_degrees -= (pos.0 - last_pos.0) * 0.2;
    r.pitch_degrees -= (pos.1 - last_pos.1) * 0.2;
    r.pitch_degrees = r.pitch_degrees.clamp(-89.0, 89.0);

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
  }
  *last_pos = LastPos(pos.0, pos.1);
  if renderer.window.get_key(Key::Escape) == Action::Press {
    renderer.window.set_cursor_mode(CursorMode::Normal);
  }
  if renderer.window.get_mouse_button(MouseButton::Button1) == Action::Press {
    renderer.window.set_cursor_mode(CursorMode::Disabled);
  }
  Ok(())
}

fn pass(ui: &Ui, name: &str, query: &mut Query) {
  ui.text(format!(
    "{}: {:.4}ms",
    name,
    query.get_blocking() as f32 / 1000000.0
  ));
}

fn ui(world: &mut World) -> Result {
  let ui = world.get_resource::<Ui>().unwrap();
  ui.window("tools")
    .position([8.0, 8.0], Condition::Once)
    .size([480.0, 360.0], Condition::Once)
    .bg_alpha(0.99)
    .build(|| {
      ui.text(format!("{:.0}fps", ui.io().framerate));
      if let Some(_) = ui.tab_bar("##") {
        if let Some(_) = ui.tab_item("World") {
          Drag::new("Sun").build_array(
            ui,
            world.get_resource::<SkySettings>().unwrap().dir.as_mut(),
          );

          if ui.button("Spawn object") {
            let assets = world.get_resource::<Assets>().unwrap();
            let mut rng = rand::thread_rng();
            let (mesh, collider) = match rng.gen_range(0..=2) {
              0 => ("sphere.obj", ColliderBuilder::ball(0.5)),
              1 => ("cube.obj", ColliderBuilder::cuboid(0.5, 0.5, 0.5)),
              2 => ("cone.obj", ColliderBuilder::cone(0.5, 0.5)),
              _ => unreachable!(),
            };
            let rb = RigidBodyBuilder::dynamic().build(world);
            world
              .spawn("object")
              .insert(
                Transform::new()
                  .pos(Vec3::new(
                    rng.gen_range(-1.0..1.0),
                    rng.gen_range(5.0..7.0),
                    rng.gen_range(-1.0..1.0),
                  ))
                  .scale(Vec3::splat(0.5)),
              )
              .insert(Model::new(assets.load(mesh).unwrap()))
              .insert(collider.attach_rb(rb).build(world))
              .insert(Material {
                color: Vec3::new(
                  rng.gen_range(0.0..1.0),
                  rng.gen_range(0.0..1.0),
                  rng.gen_range(0.0..1.0),
                ),
                tex: None,
                spec: 0.5,
                metallic: 0.5,
              })
              .insert(rb);
          }

          if ui.button("Save scene") {
            Scene::save(world, "test.scene".into()).unwrap();
          }
        }
        if let Some(_) = ui.tab_item("Graphics") {
          let scene_perf = world.get_resource::<ScenePerf>().unwrap();
          pass(ui, "shadow", &mut scene_perf.shadow_pass);
          pass(ui, "geometry", &mut scene_perf.geometry_pass);
          pass(ui, "ssao", &mut scene_perf.ssao_pass);
          pass(ui, "lighting", &mut scene_perf.lighting_pass);
          let tonemap = world.get_resource::<Tonemap>().unwrap();
          if let Some(_) = ui.begin_combo("Tonemap", tonemap.name()) {
            for t in Tonemap::ALL {
              if ui.selectable(t.name()) {
                *tonemap = t;
              }
            }
          }
        }
        if let Some(_) = ui.tab_item("Physics") {
          Drag::new("Gravity").build_array(ui, world.get_resource::<Gravity>().unwrap().0.as_mut());
          let debug_pipeline = world.get_resource::<DebugRenderPipeline>().unwrap();
          let mut debug_render = debug_pipeline.mode.is_all();
          ui.checkbox("Debug Renderer", &mut debug_render);
          debug_pipeline.mode = if debug_render {
            DebugRenderMode::all()
          } else {
            DebugRenderMode::empty()
          }
        }
        if let Some(_) = ui.tab_item("Profiler") {
          world.get_resource::<ProfilerUi>().unwrap().ui(ui);
        }
      }
    });
  Ok(())
}
