use phosphor::Result;
use phosphor::ecs::{World, Name, Stage};
use phosphor::gfx::{Texture, Mesh, Framebuffer, gl};
use phosphor::math::Vec3;
use phosphor_ui::Textures;
use phosphor_ui::imgui::{Ui, Image, TextureId, WindowFlags};
use phosphor_3d::{Camera, Transform, SceneDrawOptions, scenerenderer};
use crate::SelectedEntity;

pub struct Panel {
  pub title: &'static str,
  pub flags: WindowFlags,
  pub open: bool,
  pub render: &'static dyn Fn(&mut World, &Ui),
}

pub fn setup_panels(world: &mut World) -> Result<()> {
  let scene = scene_init(world)?;
  let outline = outline_init();
  let inspector = inspector_init();
  world.add_resource(vec![scene, outline, inspector]);
  Ok(())
}

struct SceneState {
  size: [f32; 2],
  fb: Framebuffer,
  tex: TextureId,
}

fn scene_init(world: &mut World) -> Result<Panel> {
  let textures = world.get_resource::<Textures>().unwrap();
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
    .insert(Mesh::load("res/teapot.obj")?);
  let fb = Framebuffer::new();
  let tex = Texture::empty();
  fb.bind_tex(&tex);
  let tex = textures.insert(tex);
  world.add_resource(SceneState {
    size: [0.0, 0.0],
    fb,
    tex,
  });
  scenerenderer(world)?;
  world.add_system(Stage::PreDraw, &scene_predraw);
  Ok(Panel {
    title: "Scene",
    flags: WindowFlags::NO_SCROLLBAR | WindowFlags::NO_SCROLL_WITH_MOUSE,
    open: true,
    render: &scene_render,
  })
}

fn scene_predraw(world: &mut World) -> Result {
  let s = world.get_resource::<SceneState>().unwrap();
  world.add_resource(SceneDrawOptions {
    fb: s.fb,
    size: s.size,
  });
  Ok(())
}

fn scene_render(world: &mut World, ui: &Ui) {
  let s = world.get_resource::<SceneState>().unwrap();
  s.size = ui.window_size();
  Image::new(s.tex, s.size)
    .uv0([0.0, 1.0])
    .uv1([1.0, 0.0])
    .build(&ui);
  let tex = world
    .get_resource::<Textures>()
    .unwrap()
    .get(s.tex)
    .unwrap();
  tex.resize(s.size[0] as _, s.size[1] as _);
}

fn outline_init() -> Panel {
  Panel {
    title: "Outline",
    flags: WindowFlags::empty(),
    open: true,
    render: &outline_render,
  }
}

fn outline_render(world: &mut World, ui: &Ui) {
  let [w, _] = ui.window_size();
  let selected = world.get_resource::<SelectedEntity>().unwrap();
  for (e, n) in world.query::<Name>() {
    if ui
      .selectable_config(n.0.clone())
      .selected(e.id == selected.0.unwrap_or_default())
      .build()
    {
      *selected = SelectedEntity(Some(e.id));
    }
  }
  ui.button_with_size("Add Entity", [w, 0.0]);
}

fn inspector_init() -> Panel {
  Panel {
    title: "Inspector",
    flags: WindowFlags::empty(),
    open: true,
    render: &inspector_render,
  }
}

fn inspector_render(world: &mut World, ui: &Ui) {
  match world.get_resource::<SelectedEntity>().unwrap().0 {
    Some(e) => {
      let size = ui.content_region_avail();
      let (e, n) = world.get_id::<Name>(e).unwrap();
      let mut buf = n.0.clone();
      ui.set_next_item_width(size[0]);
      if ui
        .input_text("##", &mut buf)
        .enter_returns_true(true)
        .build()
        && !buf.is_empty()
      {
        *n = Name(buf);
      }
    }
    None => ui.text("no entity selected."),
  }
}
