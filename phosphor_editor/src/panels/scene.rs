use phosphor::Result;
use phosphor::ecs::{World, Name, Stage};
use phosphor::gfx::{Texture, Mesh, Framebuffer, Renderer};
use phosphor::glfw::{Key, Action, CursorMode, MouseButton};
use phosphor::math::{Vec3, Quat};
use phosphor::assets::Assets;
use phosphor_ui::imgui::{Ui, Image, TextureId, WindowFlags, StyleVar};
use phosphor_3d::{Camera, Transform, Material, SceneDrawOptions, scenerenderer};
use crate::SelectedEntity;
use crate::panels::Panel;

struct SceneState {
  size: [f32; 2],
  focused: bool,
  fb: Framebuffer,
  tex: Texture,
  last_pos: (f32, f32),
  yaw: f32,
  pitch: f32,
}

pub fn init(world: &mut World) -> Result<Panel> {
  let assets = world.get_resource::<Assets>().unwrap();
  world
    .spawn("cam")
    .insert(Transform::new().pos(Vec3::new(0.0, 1.0, -10.0)))
    .insert(Camera::new(80.0, [0.1, 100.0]));
  world
    .spawn("teapot")
    .insert(Transform::new())
    .insert(assets.load::<Mesh>("res/teapot.obj")?)
    .insert(Material::texture(assets.load("res/brick.jpg")?));
  world
    .spawn("cylinder")
    .insert(Transform::new().pos(Vec3::new(5.0, 2.0, 0.0)))
    .insert(assets.load::<Mesh>("res/cylinder.obj")?)
    .insert(Material::color(Vec3::X));
  world
    .spawn("garfield")
    .insert(
      Transform::new()
        .pos(Vec3::new(-5.0, 0.0, 0.0))
        .scale(Vec3::splat(5.0)),
    )
    .insert(assets.load::<Mesh>("res/garfield.obj")?)
    .insert(Material::texture(assets.load("res/garfield.png")?));
  let fb = Framebuffer::new();
  let tex = Texture::empty();
  fb.bind_tex(&tex);
  world.add_resource(SceneState {
    size: [0.0, 0.0],
    focused: false,
    fb,
    tex,
    last_pos: (0.0, 0.0),
    yaw: 1.5,
    pitch: 0.0,
  });
  scenerenderer(world)?;
  world.add_system(Stage::PreDraw, &predraw);
  Ok(Panel {
    title: "\u{e1c3} Scene",
    flags: WindowFlags::NO_SCROLLBAR | WindowFlags::NO_SCROLL_WITH_MOUSE,
    vars: &[StyleVar::WindowPadding([0.0, 0.0])],
    open: true,
    render: &render,
  })
}

fn predraw(world: &mut World) -> Result {
  let renderer = world.get_resource::<Renderer>().unwrap();
  let s = world.get_resource::<SceneState>().unwrap();
  if s.focused {
    // todo show warning if no camera
    let (e, cam) = &world.query::<Camera>()[0];
    let cam_t = e.get::<Transform>().unwrap();

    if renderer.window.get_mouse_button(MouseButton::Button1) == Action::Press {
      let pos = renderer.window.get_cursor_pos();
      let (x, y) = (pos.0 as _, pos.1 as _);
      if renderer.window.get_cursor_mode() != CursorMode::Disabled {
        s.last_pos = (x, y);
        renderer.window.set_cursor_mode(CursorMode::Disabled);
      }
      let (dx, dy) = (x - s.last_pos.0, y - s.last_pos.1);
      s.last_pos = (x, y);
      s.yaw += dx / 300.0;
      s.pitch = (s.pitch - dy / 300.0).clamp(-1.5, 1.5);
    } else {
      renderer.window.set_cursor_mode(CursorMode::Normal);
    }

    let dir = Vec3::new(
      s.yaw.cos() * s.pitch.cos(),
      s.pitch.sin(),
      s.yaw.sin() * s.pitch.cos(),
    ) * 0.15;
    cam_t.rotation = Quat::from_scaled_axis(dir);
    // let dir = cam_t.rotation.to_scaled_axis() * 0.15;
    let right = dir.cross(Vec3::Y);
    if renderer.window.get_key(Key::W) == Action::Press {
      cam_t.position += dir;
    }
    if renderer.window.get_key(Key::S) == Action::Press {
      cam_t.position -= dir;
    }
    if renderer.window.get_key(Key::A) == Action::Press {
      cam_t.position -= right
    }
    if renderer.window.get_key(Key::D) == Action::Press {
      cam_t.position += right;
    }
  } else {
    renderer.window.set_cursor_mode(CursorMode::Normal);
  }
  world.add_resource(SceneDrawOptions {
    fb: s.fb,
    size: s.size,
  });
  Ok(())
}

fn render(world: &mut World, ui: &Ui) {
  let s = world.get_resource::<SceneState>().unwrap();
  let selected = world.get_resource::<SelectedEntity>().unwrap();
  s.size = ui.window_size();
  s.focused = ui.is_window_focused();
  Image::new(TextureId::new(s.tex.0 as _), s.size)
    .uv0([0.0, 1.0])
    .uv1([1.0, 0.0])
    .build(&ui);
  ui.set_cursor_pos([16.0, 32.0]);
  match selected.0 {
    Some(e) => {
      let (e, n) = world.get_id::<Name>(e).unwrap();
      ui.text(format!("{}({})", n.0, e.id));
    }
    None => ui.text("No entity selected."),
  };
  ui.set_cursor_pos([16.0, 52.0]);
  ui.text(format!("{:.1}fps", ui.io().framerate));
  s.tex.resize(s.size[0] as _, s.size[1] as _);
  s.fb.resize(s.size[0] as _, s.size[1] as _);
}
