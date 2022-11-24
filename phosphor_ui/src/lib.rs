use std::{fs, path};
use std::time::Instant;
use imgui::{
  StyleColor, ConfigFlags, MouseCursor, BackendFlags, Key, FontConfig, FontGlyphRanges, TextureId,
  sys,
};
use phosphor::glfw::{
  Cursor, StandardCursor, CursorMode, WindowEvent, Action, Modifiers, MouseButton, Key as GlfwKey,
};
use phosphor::Result;
use phosphor::gfx::{Renderer, Shader, Texture, gl};
use phosphor::ecs::{World, Stage};
use phosphor::math::Mat4;
use phosphor::log::info;

pub use imgui;

pub struct UiRendererOptions {
  pub docking: bool,
  pub ini_path: Option<&'static str>,
  pub fonts: &'static [&'static [(&'static str, f32, Option<&'static [u32]>)]],
}

impl UiRendererOptions {
  const DEFAULT: Self = Self {
    docking: false,
    ini_path: None,
    fonts: &[&[("res/roboto.ttf", 16.0, None)]],
  };
}

struct UiRenderer {
  imgui: imgui::Context,
  shader: Shader,
  vert_arr: u32,
  vert_buf: u32,
  idx_buf: u32,
  last_frame: Instant,
}

pub fn uirenderer(world: &mut World) -> Result<()> {
  let renderer = world.get_resource::<Renderer>().unwrap();
  let mut imgui = imgui::Context::create();
  info!("Created imgui {} context.", imgui::dear_imgui_version());
  let options = match world.get_resource::<UiRendererOptions>() {
    Some(o) => o,
    None => &UiRendererOptions::DEFAULT,
  };
  imgui.set_ini_filename(options.ini_path.map(|s| path::PathBuf::from(s)));
  let io = imgui.io_mut();
  if options.docking {
    io.config_flags |= ConfigFlags::DOCKING_ENABLE;
  }
  let (w, h) = renderer.window.get_size();
  let (scale_w, scale_h) = renderer.window.get_content_scale();
  io.display_size = [w as _, h as _];
  io.display_framebuffer_scale = [scale_w, scale_h];
  io.backend_flags.insert(BackendFlags::HAS_MOUSE_CURSORS);
  io.backend_flags.insert(BackendFlags::HAS_SET_MOUSE_POS);
  io[Key::Tab] = GlfwKey::Tab as _;
  io[Key::LeftArrow] = GlfwKey::Left as _;
  io[Key::RightArrow] = GlfwKey::Right as _;
  io[Key::UpArrow] = GlfwKey::Up as _;
  io[Key::DownArrow] = GlfwKey::Down as _;
  io[Key::PageUp] = GlfwKey::PageUp as _;
  io[Key::PageDown] = GlfwKey::PageDown as _;
  io[Key::Home] = GlfwKey::Home as _;
  io[Key::End] = GlfwKey::End as _;
  io[Key::Insert] = GlfwKey::Insert as _;
  io[Key::Delete] = GlfwKey::Delete as _;
  io[Key::Backspace] = GlfwKey::Backspace as _;
  io[Key::Space] = GlfwKey::Space as _;
  io[Key::Enter] = GlfwKey::Enter as _;
  io[Key::Escape] = GlfwKey::Escape as _;
  io[Key::KeyPadEnter] = GlfwKey::KpEnter as _;
  io[Key::A] = GlfwKey::A as _;
  io[Key::C] = GlfwKey::C as _;
  io[Key::V] = GlfwKey::V as _;
  io[Key::X] = GlfwKey::X as _;
  io[Key::Y] = GlfwKey::Y as _;
  io[Key::Z] = GlfwKey::Z as _;

  let mut fonts = imgui.fonts();
  for font in options.fonts {
    fonts.add_font(
      &font
        .iter()
        .map(|f| imgui::FontSource::TtfData {
          data: Box::leak(fs::read(f.0).unwrap().into_boxed_slice()),
          size_pixels: f.1,
          config: f.2.map(|g| FontConfig {
            glyph_ranges: FontGlyphRanges::from_slice(g),
            ..FontConfig::default()
          }),
        })
        .collect::<Vec<_>>(),
    );
  }
  let font_tex = fonts.build_rgba32_texture();
  fonts.tex_id =
    TextureId::new(Texture::new(font_tex.data, font_tex.width, font_tex.height).0 as _);
  let style = imgui.style_mut();
  style[StyleColor::Text] = [1.00, 1.00, 1.00, 1.00];
  style[StyleColor::TextDisabled] = [0.50, 0.50, 0.50, 1.00];
  style[StyleColor::WindowBg] = [0.10, 0.10, 0.10, 1.00];
  style[StyleColor::ChildBg] = [0.00, 0.00, 0.00, 0.00];
  style[StyleColor::PopupBg] = [0.19, 0.19, 0.19, 0.92];
  style[StyleColor::Border] = [0.19, 0.19, 0.19, 0.29];
  style[StyleColor::BorderShadow] = [0.00, 0.00, 0.00, 0.24];
  style[StyleColor::FrameBg] = [0.05, 0.05, 0.05, 0.54];
  style[StyleColor::FrameBgHovered] = [0.19, 0.19, 0.19, 0.54];
  style[StyleColor::FrameBgActive] = [0.20, 0.22, 0.23, 1.00];
  style[StyleColor::TitleBg] = [0.00, 0.00, 0.00, 1.00];
  style[StyleColor::TitleBgActive] = [0.06, 0.06, 0.06, 1.00];
  style[StyleColor::TitleBgCollapsed] = [0.00, 0.00, 0.00, 1.00];
  style[StyleColor::MenuBarBg] = [0.14, 0.14, 0.14, 1.00];
  style[StyleColor::ScrollbarBg] = [0.05, 0.05, 0.05, 0.54];
  style[StyleColor::ScrollbarGrab] = [0.34, 0.34, 0.34, 0.54];
  style[StyleColor::ScrollbarGrabHovered] = [0.40, 0.40, 0.40, 0.54];
  style[StyleColor::ScrollbarGrabActive] = [0.56, 0.56, 0.56, 0.54];
  style[StyleColor::CheckMark] = [0.33, 0.67, 0.86, 1.00];
  style[StyleColor::SliderGrab] = [0.34, 0.34, 0.34, 0.54];
  style[StyleColor::SliderGrabActive] = [0.56, 0.56, 0.56, 0.54];
  style[StyleColor::Button] = [0.05, 0.05, 0.05, 0.54];
  style[StyleColor::ButtonHovered] = [0.19, 0.19, 0.19, 0.54];
  style[StyleColor::ButtonActive] = [0.20, 0.22, 0.23, 1.00];
  style[StyleColor::Header] = [0.00, 0.00, 0.00, 0.52];
  style[StyleColor::HeaderHovered] = [0.00, 0.00, 0.00, 0.36];
  style[StyleColor::HeaderActive] = [0.20, 0.22, 0.23, 0.33];
  style[StyleColor::Separator] = [0.28, 0.28, 0.28, 0.29];
  style[StyleColor::SeparatorHovered] = [0.44, 0.44, 0.44, 0.29];
  style[StyleColor::SeparatorActive] = [0.40, 0.44, 0.47, 1.00];
  style[StyleColor::ResizeGrip] = [0.28, 0.28, 0.28, 0.29];
  style[StyleColor::ResizeGripHovered] = [0.44, 0.44, 0.44, 0.29];
  style[StyleColor::ResizeGripActive] = [0.40, 0.44, 0.47, 1.00];
  style[StyleColor::Tab] = [0.00, 0.00, 0.00, 0.52];
  style[StyleColor::TabHovered] = [0.14, 0.14, 0.14, 1.00];
  style[StyleColor::TabActive] = [0.20, 0.20, 0.20, 0.36];
  style[StyleColor::TabUnfocused] = [0.00, 0.00, 0.00, 0.52];
  style[StyleColor::TabUnfocusedActive] = [0.14, 0.14, 0.14, 1.00];
  style[StyleColor::DockingPreview] = [0.33, 0.67, 0.86, 1.00];
  style[StyleColor::DockingEmptyBg] = [0.10, 0.10, 0.10, 1.00];
  style[StyleColor::PlotLines] = [1.00, 0.00, 0.00, 1.00];
  style[StyleColor::PlotLinesHovered] = [1.00, 0.00, 0.00, 1.00];
  style[StyleColor::PlotHistogram] = [1.00, 0.00, 0.00, 1.00];
  style[StyleColor::PlotHistogramHovered] = [1.00, 0.00, 0.00, 1.00];
  style[StyleColor::TableHeaderBg] = [0.00, 0.00, 0.00, 0.52];
  style[StyleColor::TableBorderStrong] = [0.00, 0.00, 0.00, 0.52];
  style[StyleColor::TableBorderLight] = [0.28, 0.28, 0.28, 0.29];
  style[StyleColor::TableRowBg] = [0.00, 0.00, 0.00, 0.00];
  style[StyleColor::TableRowBgAlt] = [1.00, 1.00, 1.00, 0.06];
  style[StyleColor::TextSelectedBg] = [0.20, 0.22, 0.23, 1.00];
  style[StyleColor::DragDropTarget] = [0.33, 0.67, 0.86, 1.00];
  style[StyleColor::NavHighlight] = [0.05, 0.05, 0.05, 0.54];
  style[StyleColor::NavWindowingHighlight] = [0.19, 0.19, 0.19, 0.54];
  style[StyleColor::NavWindowingDimBg] = [1.00, 0.00, 0.00, 0.20];
  style[StyleColor::ModalWindowDimBg] = [1.00, 0.00, 0.00, 0.35];
  style.window_rounding = 4.0;
  style.popup_rounding = 4.0;
  style.frame_rounding = 2.0;

  let shader = Shader::new("res/imgui.vert", "res/imgui.frag")?;
  let mut vert_arr = 0;
  let mut vert_buf = 0;
  let mut idx_buf = 0;
  unsafe {
    gl::GenVertexArrays(1, &mut vert_arr);
    gl::BindVertexArray(vert_arr);
    gl::GenBuffers(1, &mut vert_buf);
    gl::BindBuffer(gl::ARRAY_BUFFER, vert_buf);
    gl::GenBuffers(1, &mut idx_buf);
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, idx_buf);
    gl::EnableVertexAttribArray(0);
    gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 20, 0 as _);
    gl::EnableVertexAttribArray(1);
    gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 20, 8 as _);
    gl::EnableVertexAttribArray(2);
    gl::VertexAttribPointer(2, 4, gl::UNSIGNED_BYTE, gl::TRUE, 20, 16 as _);
  }
  world.add_resource(UiRenderer {
    imgui,
    shader,
    vert_arr,
    vert_buf,
    idx_buf,
    last_frame: Instant::now(),
  });
  world.add_event_handler(&uirenderer_event);
  world.add_system(Stage::PreDraw, &uirenderer_predraw);
  world.add_system(Stage::PostDraw, &uirenderer_draw);
  Ok(())
}

fn uirenderer_event(world: &mut World, event: &WindowEvent) -> Result<()> {
  let r = world.get_resource::<UiRenderer>().unwrap();
  let io = r.imgui.io_mut();
  match *event {
    WindowEvent::Key(key, _scancode, action, modifiers) => {
      if key as i32 >= 0 {
        if action == Action::Release {
          io.keys_down[key as usize] = false;
        } else {
          io.keys_down[key as usize] = true;
        }
      }
      io.key_shift = modifiers.contains(Modifiers::Shift);
      io.key_ctrl = modifiers.contains(Modifiers::Control);
      io.key_alt = modifiers.contains(Modifiers::Alt);
      io.key_super = modifiers.contains(Modifiers::Super);
    }
    WindowEvent::Size(width, height) => {
      io.display_size = [width as _, height as _];
    }
    WindowEvent::Char(ch) => {
      if ch != '\u{7f}' {
        io.add_input_character(ch);
      }
    }
    WindowEvent::CursorPos(x, y) => {
      io.mouse_pos = [x as _, y as _];
    }
    WindowEvent::Scroll(x, y) => {
      io.mouse_wheel_h = x as _;
      io.mouse_wheel = y as _;
    }
    WindowEvent::MouseButton(button, action, _modifiers) => {
      let pressed = action == Action::Press;
      match button {
        MouseButton::Button1 => io.mouse_down[0] = pressed,
        MouseButton::Button2 => io.mouse_down[1] = pressed,
        MouseButton::Button3 => io.mouse_down[2] = pressed,
        _ => (),
      }
    }
    _ => {}
  }
  Ok(())
}

fn uirenderer_predraw(world: &mut World) -> Result<()> {
  let renderer = world.get_resource::<Renderer>().unwrap();
  let r = world.get_resource::<UiRenderer>().unwrap();
  let io = r.imgui.io();
  if io.want_set_mouse_pos {
    let [x, y] = io.mouse_pos;
    renderer.window.set_cursor_pos(x as _, y as _);
  }
  let ui = r.imgui.frame();

  let options = match world.get_resource::<UiRendererOptions>() {
    Some(o) => o,
    None => &UiRendererOptions::DEFAULT,
  };
  if options.docking {
    unsafe {
      sys::igDockSpaceOverViewport(imgui::sys::igGetMainViewport(), 0, std::ptr::null());
    }
  }
  world.add_resource::<imgui::Ui>(unsafe { (ui as *const imgui::Ui).read() });
  Ok(())
}

fn uirenderer_draw(world: &mut World) -> Result<()> {
  if let Some(ui) = world.take_resource::<imgui::Ui>() {
    let renderer = world.get_resource::<Renderer>().unwrap();
    let r = world.get_resource::<UiRenderer>().unwrap();
    unsafe {
      gl::Disable(gl::DEPTH_TEST);
      gl::BindVertexArray(r.vert_arr);
      let io = r.imgui.io_mut();
      let now = Instant::now();
      io.update_delta_time(now - r.last_frame);
      r.last_frame = now;
      if !io
        .config_flags
        .contains(ConfigFlags::NO_MOUSE_CURSOR_CHANGE)
      {
        match ui.mouse_cursor() {
          Some(mouse_cursor) if !io.mouse_draw_cursor => {
            // renderer.window.set_cursor_mode(CursorMode::Normal);
            renderer.window.set_cursor(Some(match mouse_cursor {
              MouseCursor::Arrow => Cursor::standard(StandardCursor::Arrow),
              MouseCursor::ResizeAll => Cursor::standard(StandardCursor::Arrow),
              MouseCursor::ResizeNS => Cursor::standard(StandardCursor::VResize),
              MouseCursor::ResizeEW => Cursor::standard(StandardCursor::HResize),
              MouseCursor::ResizeNESW => Cursor::standard(StandardCursor::Arrow),
              MouseCursor::ResizeNWSE => Cursor::standard(StandardCursor::Arrow),
              MouseCursor::Hand => Cursor::standard(StandardCursor::Hand),
              MouseCursor::NotAllowed => Cursor::standard(StandardCursor::Crosshair),
              MouseCursor::TextInput => Cursor::standard(StandardCursor::IBeam),
            }));
          }
          _ => renderer.window.set_cursor_mode(CursorMode::Hidden),
        }
      }

      let [w, h] = ui.io().display_size;
      r.shader.bind();
      r.shader.set_mat4(
        "transform",
        &Mat4::orthographic_rh(0.0, w as _, h as _, 0.0, 0.0, 1.0),
      );
      let [scale_w, scale_h] = ui.io().display_framebuffer_scale;

      let draw_data = r.imgui.render();
      for draw_list in draw_data.draw_lists() {
        gl::BindBuffer(gl::ARRAY_BUFFER, r.vert_buf);
        gl::BufferData(
          gl::ARRAY_BUFFER,
          (draw_list.vtx_buffer().len() * 20) as _,
          draw_list.vtx_buffer().as_ptr() as _,
          gl::DYNAMIC_DRAW,
        );
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, r.idx_buf);
        gl::BufferData(
          gl::ELEMENT_ARRAY_BUFFER,
          (draw_list.idx_buffer().len() * 2) as _,
          draw_list.idx_buffer().as_ptr() as _,
          gl::DYNAMIC_DRAW,
        );
        for cmd in draw_list.commands() {
          if let imgui::DrawCmd::Elements { count, cmd_params } = cmd {
            Texture(cmd_params.texture_id.id() as _).bind();
            gl::Scissor(
              (cmd_params.clip_rect[0] * scale_w) as _,
              (h * scale_h - cmd_params.clip_rect[3] * scale_h) as _,
              ((cmd_params.clip_rect[2] - cmd_params.clip_rect[0]) * scale_w) as _,
              ((cmd_params.clip_rect[3] - cmd_params.clip_rect[1]) * scale_h) as _,
            );
            gl::DrawElements(
              gl::TRIANGLES,
              count as _,
              gl::UNSIGNED_SHORT,
              (cmd_params.idx_offset * 2) as _,
            );
          }
        }
      }
      gl::Enable(gl::DEPTH_TEST);
    }
  }
  Ok(())
}

pub fn hover_tooltip(ui: &imgui::Ui, text: &'static str) {
  if ui.is_item_hovered() {
    ui.tooltip_text(text);
  }
}
