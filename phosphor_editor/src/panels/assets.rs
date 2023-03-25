use std::any::Any;
use std::collections::HashMap;
use phosphor::TypeIdNamed;
use phosphor::gfx::{Texture, Mesh, Shader, Framebuffer, Renderer, gl};
use phosphor::ecs::World;
use phosphor::assets::{Assets, Handle};
use phosphor::math::{Mat4, Vec3, Quat};
use phosphor_imgui::imgui::{Ui, WindowFlags, Image, TextureId};
use crate::panels::Panel;

type Preview = fn(&Ui, &World, &Handle<dyn Any>, [f32; 2]);

pub struct SelectedAsset(pub Option<(TypeIdNamed, Handle<dyn Any>)>);

struct MeshPreviewState {
  fb: Framebuffer,
  textures: HashMap<String, Texture>,
  selected_tex: Texture,
  shader: Shader,
  spin: f32,
}

pub fn init(world: &mut World) -> Panel {
  let mut previews = HashMap::new();
  previews.insert(TypeIdNamed::of::<Texture>(), preview_texture as Preview);
  previews.insert(TypeIdNamed::of::<Mesh>(), preview_mesh);
  world.add_resource(previews);
  world.add_resource(SelectedAsset(None));
  let fb = Framebuffer::new();
  world.add_resource(MeshPreviewState {
    fb,
    textures: HashMap::new(),
    selected_tex: Texture::empty(),
    shader: Shader::new("base.vert", "unlit.frag").unwrap(),
    spin: 0.0,
  });
  Panel {
    title: "\u{f660} Assets",
    flags: WindowFlags::empty(),
    vars: &[],
    open: true,
    render,
  }
}

fn preview_texture(ui: &Ui, _: &World, handle: &Handle<dyn Any>, size: [f32; 2]) {
  let tex = handle.downcast::<Texture>();
  let short = size[0].min(size[1]);
  Image::new(TextureId::new(tex.id as _), [short, short]).build(ui);
  corner_info(ui, size, format!("{}x{}", tex.width, tex.height));
}

fn preview_mesh(ui: &Ui, world: &World, handle: &Handle<dyn Any>, size: [f32; 2]) {
  let state = world.get_resource::<MeshPreviewState>().unwrap();
  let renderer = world.get_resource::<Renderer>().unwrap();
  let fb_size = [size[0] * 2.5, size[1] * 2.5];
  let (tex, spin) = if size[0] == size[1] {
    (
      state
        .textures
        .entry(handle.name.clone())
        .or_insert_with(Texture::empty),
      0.0,
    )
  } else {
    state.spin += 0.005;
    (&mut state.selected_tex, state.spin)
  };

  tex.resize(fb_size[0] as _, fb_size[1] as _);
  state.fb.resize(fb_size[0] as _, fb_size[1] as _);
  state.fb.bind_tex(tex);
  renderer.resize(fb_size[0] as _, fb_size[1] as _);
  renderer.clear(0.0, 0.0, 0.0, 0.0);
  state.shader.bind();
  state.shader.set_mat4(
    "model",
    &Mat4::from_rotation_translation(Quat::from_rotation_y(spin), Vec3::NEG_Y),
  );
  state.shader.set_mat4(
    "view",
    &Mat4::look_at_rh(Vec3::splat(5.0), Vec3::ZERO, Vec3::Y),
  );
  state.shader.set_mat4(
    "projection",
    &Mat4::perspective_rh(1.0, size[0] / size[1], 0.1, 50.0),
  );
  state.shader.set_vec3("color", &Vec3::splat(0.5));
  let mesh = handle.downcast::<Mesh>();
  mesh.draw();
  unsafe {
    gl::LineWidth(5.0);
    gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
    state.shader.set_vec3("color", &Vec3::ZERO);
    handle.downcast::<Mesh>().draw();
    gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
  }
  Image::new(TextureId::new(tex.id as _), size)
    .uv0([0.0, 1.0])
    .uv1([1.0, 0.0])
    .build(ui);
  corner_info(ui, size, format!("Verts: {}", mesh.indices.len()));
}

fn render(world: &mut World, ui: &Ui) {
  let assets = world.get_resource::<Assets>().unwrap();
  let previews = world
    .get_resource::<HashMap<TypeIdNamed, Preview>>()
    .unwrap();
  let selected = world.get_resource::<SelectedAsset>().unwrap();
  for (t, v) in assets.handles.iter() {
    let mut pos = ui.cursor_pos();
    for handle in v {
      let id = ui.push_id(handle.name.clone());
      if ui
        .selectable_config("##")
        .size([100.0, 100.0])
        .selected(
          selected
            .0
            .as_ref()
            .map(|h| h.1.name == handle.name)
            .unwrap_or(false),
        )
        .build()
      {
        *selected = SelectedAsset(Some((*t, handle.clone())));
      }
      if let Some(_) = ui.drag_drop_source_config(t.name).begin() {
        *selected = SelectedAsset(Some((*t, handle.clone())));
        ui.text(handle.name.clone());
      }
      ui.set_cursor_pos([pos[0] + 18.0, pos[1] + 8.0]);
      match previews.get(t) {
        Some(p) => (p)(ui, world, handle, [64.0, 64.0]),
        None => {
          let font = ui.push_font(ui.fonts().fonts()[1]);
          ui.text(" \u{f15b}");
          font.pop();
        }
      }
      ui.set_cursor_pos([pos[0] + 8.0, pos[1] + 76.0]);
      ui.text(handle.name.clone());
      pos[0] += 108.0;
      ui.set_cursor_pos(pos);
      id.pop();
    }
  }
  let [w, h] = ui.window_size();
  ui.set_cursor_pos([w - 320.0, 24.0]);
  ui.child_window("##")
    .border(true)
    .build(|| match &selected.0 {
      Some(handle) => {
        let font = ui.push_font(ui.fonts().fonts()[1]);
        ui.text("\u{f15b}");
        font.pop();
        ui.same_line();
        let pos = ui.cursor_pos();
        ui.text(handle.1.name.clone());
        ui.set_cursor_pos([pos[0], pos[1] + 16.0]);
        ui.text_disabled(handle.0.name);
        ui.set_cursor_pos([8.0, pos[1] + 54.0]);
        ui.separator();
        match previews.get(&handle.0) {
          Some(p) => {
            ui.text("Preview:");
            (p)(ui, world, &handle.1, [296.0, h - 128.0]);
          }
          None => ui.text("\u{f071} No preview available."),
        }
      }
      None => ui.text("\u{f071} No asset selected."),
    });
}

fn corner_info(ui: &Ui, size: [f32; 2], info: String) {
  if size[0] != size[1] {
    let [w, h] = ui.content_region_max();
    let [x, y] = ui.calc_text_size(info.clone());
    ui.set_cursor_pos([w - x, h - y]);
    ui.text(info);
  }
}
