use phosphor::{Engine, Result, grr};
use phosphor::ecs::{Stage, World, Name};
use phosphor::log::LevelFilter;
use phosphor::gfx::{Texture, Renderer, Mesh};
use phosphor::math::Vec3;
use phosphor_ui::{uirenderer, Textures, UiRendererOptions};
use phosphor_ui::imgui::{Ui, Image, TextureId};
use phosphor_3d::{
  scenerenderer, scenerenderer_draw, Camera, Transform, SceneRendererOptions, SceneAspect,
};

struct Scene {
  open: bool,
  fb: grr::Framebuffer,
  tex: TextureId,
  size: [f32; 2],
}

struct Outline {
  open: bool,
}

fn main() -> Result<()> {
  env_logger::builder().filter_level(LevelFilter::Info).init();
  Engine::new()
    .add_resource(SceneRendererOptions { draw_stage: false })
    .add_resource(UiRendererOptions {
      docking: true,
      ini_path: Some("phosphor_editor/ui.ini"),
    })
    .add_resource(Outline { open: true })
    .add_system(Stage::Start, &uirenderer)
    .add_system(Stage::Start, &scenerenderer)
    .add_system(Stage::Start, &setup)
    .add_system(Stage::Start, &setup_scene)
    .add_system(Stage::PreDraw, &draw_scene)
    .add_system(Stage::Draw, &draw_ui)
    .run()
}

fn setup_scene(world: &mut World) -> Result<()> {
  let renderer = world.get_resource::<Renderer>().unwrap();
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
    .insert(Mesh::load(renderer, "res/teapot.obj")?);
  Ok(())
}

fn setup(world: &mut World) -> Result<()> {
  let renderer = world.get_resource::<Renderer>().unwrap();
  let textures = world.get_resource::<Textures>().unwrap();
  unsafe {
    let fb = renderer.gl.create_framebuffer()?;
    let tex = Texture::empty(renderer, 1280, 720)?; // todo resize
    renderer.gl.bind_attachments(
      fb,
      &[(
        grr::Attachment::Color(0),
        grr::AttachmentView::Image(tex.view()),
      )],
    );
    world.add_resource(Scene {
      open: true,
      fb,
      tex: textures.insert(tex),
      size: [1280.0, 720.0],
    })
  }
  Ok(())
}

fn draw_scene(world: &mut World) -> Result<()> {
  let scene = world.get_resource::<Scene>().unwrap();
  if scene.open {
    let renderer = world.get_resource::<Renderer>().unwrap();
    unsafe {
      renderer.gl.bind_framebuffer(scene.fb);
      renderer.resize([1280.0, 720.0]);
      renderer.clear(scene.fb);
    }
    world.add_resource(SceneAspect(scene.size[0] / scene.size[1]));
    scenerenderer_draw(world)?;
  }
  Ok(())
}

fn draw_ui(world: &mut World) -> Result<()> {
  let scene = world.get_resource::<Scene>().unwrap();
  let outline = world.get_resource::<Outline>().unwrap();
  let ui = world.get_resource::<Ui>().unwrap();
  ui.main_menu_bar(|| {
    ui.menu("View", || {
      ui.menu_item_config("Scene").build_with_ref(&mut scene.open);
      ui.menu_item_config("Outline")
        .build_with_ref(&mut outline.open);
    });
  });
  if scene.open {
    ui.window("Scene")
      .opened(&mut scene.open)
      .scroll_bar(false)
      .scrollable(false)
      .build(|| {
        Image::new(scene.tex, scene.size)
          .uv0([0.0, 1.0])
          .uv1([1.0, 0.0])
          .build(&ui);
        scene.size = ui.window_size();
      });
  }
  if outline.open {
    ui.window("Outline").opened(&mut outline.open).build(|| {
      let [w, _] = ui.window_size();
      for (_, n) in world.query::<Name>() {
        ui.selectable(n.0);
      }
      ui.button_with_size("Add Entity", [w, 0.0]);
    });
  }
  Ok(())
}
