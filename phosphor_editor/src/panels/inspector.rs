use std::collections::HashMap;
use std::any::Any;
use phosphor::{TypeIdNamed, mutate};
use phosphor::ecs::{World, Name};
use phosphor::gfx::{Texture, Mesh};
use phosphor::assets::{Handle, Assets};
use phosphor::math::Vec3;
use phosphor_ui::hover_tooltip;
use phosphor_ui::imgui::{Ui, Drag, WindowFlags, TreeNodeFlags};
use phosphor_3d::{Camera, Transform, Material};
use crate::SelectedEntity;
use crate::panels::Panel;

pub fn init(world: &mut World) -> Panel {
  let mut panels = HashMap::new();
  panels.insert(
    TypeIdNamed::of::<Name>(),
    InspectorPanel {
      label: "\u{e1cd} Name",
      render: &inspector_name,
    },
  );
  panels.insert(
    TypeIdNamed::of::<Transform>(),
    InspectorPanel {
      label: "\u{f047} Transform",
      render: &inspector_transform,
    },
  );
  panels.insert(
    TypeIdNamed::of::<Camera>(),
    InspectorPanel {
      label: "\u{f030} Camera",
      render: &inspector_camera,
    },
  );
  panels.insert(
    TypeIdNamed::of::<Handle<Mesh>>(),
    InspectorPanel {
      label: "\u{f1b2} Mesh",
      render: &inspector_mesh,
    },
  );
  panels.insert(
    TypeIdNamed::of::<Material>(),
    InspectorPanel {
      label: "\u{f5c3} Material",
      render: &inspector_material,
    },
  );
  world.add_resource(panels);
  world.add_resource(HashMap::from([
    (
      0usize,
      MaterialInspector {
        name: "Color",
        render: &material_color,
        default: &material_color_default,
      },
    ),
    (
      1,
      MaterialInspector {
        name: "Texture",
        render: &material_texture,
        default: &material_texture_default,
      },
    ),
  ]));
  Panel {
    title: "\u{f30f} Inspector",
    flags: WindowFlags::empty(),
    vars: &[],
    open: true,
    render: &render,
  }
}

struct InspectorPanel {
  pub label: &'static str,
  pub render: &'static dyn Fn(&mut Box<dyn Any>, &Ui, &mut World),
}

fn inspector_name(t: &mut Box<dyn Any>, ui: &Ui, _: &mut World) {
  let name: &mut Name = t.downcast_mut().unwrap();
  let mut buf = name.0.clone();
  let size = ui.content_region_avail();
  ui.set_next_item_width(size[0]);
  if ui
    .input_text("##", &mut buf)
    .enter_returns_true(true)
    .build()
    && !buf.is_empty()
  {
    *name = Name(buf);
  }
}
fn inspector_transform(t: &mut Box<dyn Any>, ui: &Ui, _: &mut World) {
  let transform: &mut Transform = t.downcast_mut().unwrap();
  Drag::new("position")
    .speed(0.05)
    .display_format("%g")
    .build_array(ui, transform.position.as_mut());
  Drag::new("rotation")
    .speed(0.5)
    .display_format("%g")
    .build_array(ui, transform.rotation.as_mut());
  Drag::new("scale")
    .speed(0.05)
    .display_format("%g")
    .build_array(ui, transform.scale.as_mut());
}

fn inspector_camera(t: &mut Box<dyn Any>, ui: &Ui, _: &mut World) {
  let cam: &mut Camera = t.downcast_mut().unwrap();
  Drag::new("fov")
    .display_format("%g°")
    .range(10.0, 180.0)
    .build(ui, &mut cam.fov);
  Drag::new("clip")
    .speed(0.05)
    .display_format("%g")
    .build_array(ui, &mut cam.clip);
}

fn inspector_mesh(t: &mut Box<dyn Any>, ui: &Ui, world: &mut World) {
  let mesh: &mut Handle<Mesh> = t.downcast_mut().unwrap();
  asset_picker(ui, "mesh", world.get_resource::<Assets>().unwrap(), mesh);
}

struct MaterialInspector {
  name: &'static str,
  render: &'static dyn Fn(&Ui, &mut Material, &mut World),
  default: &'static dyn Fn(&mut World) -> Material,
}

fn inspector_material(t: &mut Box<dyn Any>, ui: &Ui, world: &mut World) {
  let mat: &mut Material = t.downcast_mut().unwrap();
  let mats = world
    .get_resource::<HashMap<usize, MaterialInspector>>()
    .unwrap();
  let mat_i = mats.get(&mat.id);
  let id = ui.push_id("##");
  if let Some(_) = ui.begin_combo("type", mat_i.map(|m| m.name).unwrap_or("??")) {
    for (t, i) in mats.iter() {
      if ui.selectable_config(i.name).selected(*t == mat.id).build() {
        *mat = (i.default)(world);
        return;
      }
    }
  }
  match mat_i {
    Some(s) => (s.render)(ui, mat, world),
    None => ui.text(format!("\u{f071} Unknown material '{}'.", mat.id)),
  }
  id.end();
}

fn material_color(ui: &Ui, mat: &mut Material, _: &mut World) {
  let col: &mut Vec3 = mat.data.downcast_mut().unwrap();
  ui.color_edit3("color", col.as_mut());
}

fn material_color_default(_: &mut World) -> Material {
  Material::color(Vec3::splat(0.75))
}

fn material_texture(ui: &Ui, mat: &mut Material, world: &mut World) {
  let tex: &mut Handle<Texture> = mat.data.downcast_mut().unwrap();
  asset_picker(ui, "texture", world.get_resource::<Assets>().unwrap(), tex);
}

fn material_texture_default(world: &mut World) -> Material {
  let assets = world.get_resource::<Assets>().unwrap();
  Material::texture(assets.get::<Texture>()[0].clone())
}

fn render(world: &mut World, ui: &Ui) {
  match world.get_resource::<SelectedEntity>().unwrap().0 {
    Some(e) => {
      let panels = world
        .get_resource::<HashMap<TypeIdNamed, InspectorPanel>>()
        .unwrap();

      for (t, v) in world.get_all(e) {
        match panels.get(&t) {
          Some(panel) => {
            for c in v {
              if ui.collapsing_header(panel.label, TreeNodeFlags::DEFAULT_OPEN) {
                hover_tooltip(ui, t.name);
                (panel.render)(c, ui, mutate(world));
              } else {
                hover_tooltip(ui, t.name);
              }
            }
          }
          None => ui.disabled(true, || {
            ui.collapsing_header(t.name, TreeNodeFlags::empty());
          }),
        }
      }
      ui.separator();
      let [w, _] = ui.window_size();
      ui.button_with_size("\u{2b} Add Component", [w, 0.0]);
    }
    None => ui.text("\u{f071} No entity selected."),
  }
}

fn asset_picker<T: Any>(ui: &Ui, label: &str, assets: &mut Assets, handle: &mut Handle<T>) {
  let id = ui.push_id("##");
  if let Some(_) = ui.begin_combo(label, handle.name.clone()) {
    for asset in assets.get::<T>() {
      if ui
        .selectable_config(asset.name.clone())
        .selected(handle.name == asset.name)
        .build()
      {
        *handle = asset;
      }
    }
  }
  id.end();
}
