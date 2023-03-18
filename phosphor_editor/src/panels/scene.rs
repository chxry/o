use std::f32::consts::FRAC_PI_2;
use phosphor::Result;
use phosphor::ecs::{World, Name, stage};
use phosphor::gfx::{Texture, Framebuffer, Renderer};
use phosphor::glfw::{Key, Action, CursorMode, MouseButton};
use phosphor::math::{Vec3, EulerRot, Quat};
use phosphor_imgui::imgui::{Ui, Image, TextureId, WindowFlags, StyleVar, Condition};
use phosphor_3d::{Camera, Transform, SceneDrawOptions, scenerenderer_plugin};
use crate::{SelectedEntity, load};
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
  load(world);
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
  scenerenderer_plugin(world)?;
  world.add_system(stage::PRE_DRAW, predraw);
  Ok(Panel {
    title: "\u{e1c3} Scene",
    flags: WindowFlags::NO_SCROLLBAR | WindowFlags::NO_SCROLL_WITH_MOUSE,
    vars: &[StyleVar::WindowPadding([0.0, 0.0])],
    open: true,
    render,
  })
}

fn predraw(world: &mut World) -> Result {
  let renderer = world.get_resource::<Renderer>().unwrap();
  let s = world.get_resource::<SceneState>().unwrap();
  if s.focused {
    match world.query::<Camera>().first() {
      Some((e, _)) => {
        s.cam = true;
        let cam_t = e.get_one::<Transform>().unwrap();

        if renderer.window.get_mouse_button(MouseButton::Button1) == Action::Press {
          let pos = renderer.window.get_cursor_pos();
          let (x, y) = (pos.0 as _, pos.1 as _);
          if renderer.window.get_cursor_mode() != CursorMode::Disabled {
            s.last_pos = (x, y);
            renderer.window.set_cursor_mode(CursorMode::Disabled);
          }
          let (dx, dy) = (x - s.last_pos.0, y - s.last_pos.1);
          s.last_pos = (x, y);
          let mut euler = cam_t.rotation.to_euler(EulerRot::YXZ);
          euler.0 -= dx * 0.005;
          euler.1 -= dy * 0.005;
          euler.1 = euler.1.clamp(-FRAC_PI_2 + 0.1, FRAC_PI_2 - 0.1);
          cam_t.rotation = Quat::from_euler(EulerRot::YXZ, euler.0, euler.1, euler.2);
        } else {
          renderer.window.set_cursor_mode(CursorMode::Normal);
        }

        let front = cam_t.rotation * Vec3::NEG_Z;
        let right = front.cross(Vec3::Y);
        let mut trans = Vec3::ZERO;
        if renderer.window.get_key(Key::W) == Action::Press {
          trans += front;
        }
        if renderer.window.get_key(Key::S) == Action::Press {
          trans -= front;
        }
        if renderer.window.get_key(Key::A) == Action::Press {
          trans -= right
        }
        if renderer.window.get_key(Key::D) == Action::Press {
          trans += right;
        }
        cam_t.position += trans.normalize_or_zero() * 0.1;
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
    size: [s.size[0] * 2.5, s.size[1] * 2.5],
  });
  Ok(())
}

fn render(world: &mut World, ui: &Ui) {
  let s = world.get_resource::<SceneState>().unwrap();
  let selected = world.get_resource::<SelectedEntity>().unwrap();
  s.size = ui.window_size();
  s.focused = ui.is_window_focused();
  if s.cam {
    let pos = ui.cursor_screen_pos();
    Image::new(TextureId::new(s.tex.id as _), s.size)
      .uv0([0.0, 1.0])
      .uv1([1.0, 0.0])
      .build(&ui);
    let pad = ui.push_style_var(StyleVar::WindowPadding([2.0, 2.0]));
    let round = ui.push_style_var(StyleVar::WindowRounding(0.0));
    ui.window("##")
      .flags(WindowFlags::NO_DECORATION | WindowFlags::ALWAYS_AUTO_RESIZE | WindowFlags::NO_MOVE)
      .bg_alpha(0.5)
      .position(pos, Condition::Always)
      .build(|| {
        ui.set_window_font_scale(0.8);
        ui.text(match selected.0 {
          Some(e) => e.get_one::<Name>().unwrap().0.clone(),
          None => "No entity selected.".to_string(),
        });
        ui.text(format!("{:.1}fps", ui.io().framerate));
      });
    pad.pop();
    round.pop();
  } else {
    let font = ui.push_font(ui.fonts().fonts()[1]);
    ui.set_window_font_scale(0.65);
    let msg = "\u{e0eb} No camera.";
    let [w, h] = ui.window_size();
    let [x, y] = ui.calc_text_size(msg);
    ui.set_cursor_pos([(w - x) / 2.0, (h - y) / 2.0]);
    ui.text(msg);
    ui.set_window_font_scale(1.0);
    font.pop();
  }
  s.tex.resize((2.5 * s.size[0]) as _, (2.5 * s.size[1]) as _);
  s.fb.resize((2.5 * s.size[0]) as _, (2.5 * s.size[1]) as _);
}
