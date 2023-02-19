use phosphor::Result;
use phosphor::ecs::{World, Name, stage};
use phosphor::gfx::{Texture, Framebuffer, Renderer};
use phosphor::glfw::{Key, Action, CursorMode, MouseButton};
use phosphor::math::Vec3;
use phosphor::scene::Scene;
use phosphor_imgui::imgui::{Ui, Image, TextureId, WindowFlags, StyleVar};
use phosphor_3d::{Camera, Transform, SceneDrawOptions, scenerenderer};
use crate::SelectedEntity;
use crate::panels::Panel;

struct SceneState {
  size: [f32; 2],
  focused: bool,
  cam: bool,
  fb: Framebuffer,
  tex: Texture,
  last_pos: (f32, f32),
}

pub fn init(world: &mut World) -> Result<Panel> {
  Scene::load(world, "test.scene")?;
  let fb = Framebuffer::new();
  let tex = Texture::empty();
  fb.bind_tex(&tex);
  world.add_resource(SceneState {
    size: [0.0, 0.0],
    focused: false,
    cam: false,
    fb,
    tex,
    last_pos: (0.0, 0.0),
  });
  scenerenderer(world)?;
  world.add_system(stage::PRE_DRAW, &predraw);
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
    match world.query::<Camera>().first() {
      Some((e, _)) => {
        s.cam = true;
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
          cam_t.rotation.y += dx / 5.0;
          cam_t.rotation.x = (cam_t.rotation.x - dy / 5.0).clamp(-89.9, 89.9);
        } else {
          renderer.window.set_cursor_mode(CursorMode::Normal);
        }

        let front = cam_t.dir() * 0.15;
        let right = front.cross(Vec3::Y);
        if renderer.window.get_key(Key::W) == Action::Press {
          cam_t.position += front;
        }
        if renderer.window.get_key(Key::S) == Action::Press {
          cam_t.position -= front;
        }
        if renderer.window.get_key(Key::A) == Action::Press {
          cam_t.position -= right
        }
        if renderer.window.get_key(Key::D) == Action::Press {
          cam_t.position += right;
        }
      }
      None => {
        s.cam = false;
      }
    };
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
  if s.cam {
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
  } else {
    ui.text("No camera :(");
  }
  s.tex.resize(s.size[0] as _, s.size[1] as _);
  s.fb.resize(s.size[0] as _, s.size[1] as _);
}
