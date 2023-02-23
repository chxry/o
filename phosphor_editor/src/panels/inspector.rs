use std::collections::HashMap;
use std::any::Any;
use phosphor::{TypeIdNamed, HashMapExt, mutate};
use phosphor::ecs::{World, Name};
use phosphor::assets::{Handle, Assets};
use phosphor_imgui::hover_tooltip;
use phosphor_imgui::imgui::{Ui, Drag, WindowFlags, TreeNodeFlags, DragDropFlags};
use phosphor_3d::{Camera, Transform, Model, Material};
use crate::SelectedEntity;
use crate::panels::Panel;
use super::assets::SelectedAsset;

pub fn init(world: &mut World) -> Panel {
  let mut panels = HashMap::new();
  panels.insert(
    TypeIdNamed::of::<Name>(),
    InspectorPanel {
      label: "\u{e1cd} Name",
      render: &inspector_name,
      default: &name_default,
    },
  );
  panels.insert(
    TypeIdNamed::of::<Transform>(),
    InspectorPanel {
      label: "\u{f047} Transform",
      render: &inspector_transform,
      default: &transform_default,
    },
  );
  panels.insert(
    TypeIdNamed::of::<Camera>(),
    InspectorPanel {
      label: "\u{f030} Camera",
      render: &inspector_camera,
      default: &camera_default,
    },
  );
  panels.insert(
    TypeIdNamed::of::<Model>(),
    InspectorPanel {
      label: "\u{f1b2} Model",
      render: &inspector_model,
      default: &model_default,
    },
  );
  panels.insert(
    TypeIdNamed::of::<Material>(),
    InspectorPanel {
      label: "\u{f5c3} Material",
      render: &inspector_material,
      default: &material_default,
    },
  );
  world.add_resource(panels);
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
  pub default: &'static dyn Fn(&mut World) -> Box<dyn Any>,
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

fn name_default(_: &mut World) -> Box<dyn Any> {
  Box::new(())
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

fn transform_default(_: &mut World) -> Box<dyn Any> {
  Box::new(Transform::new())
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

fn camera_default(_: &mut World) -> Box<dyn Any> {
  Box::new(Camera::new(80.0, [0.1, 100.0]))
}

fn inspector_model(t: &mut Box<dyn Any>, ui: &Ui, world: &mut World) {
  let model: &mut Model = t.downcast_mut().unwrap();
  asset_picker(ui, "mesh", world, &mut model.mesh);
  ui.checkbox("Cast Shadows", &mut model.cast_shadows);
}

fn model_default(world: &mut World) -> Box<dyn Any> {
  let assets = world.get_resource::<Assets>().unwrap();
  Box::new(Model::new(assets.load("cylinder.obj").unwrap()))
}

fn inspector_material(t: &mut Box<dyn Any>, ui: &Ui, world: &mut World) {
  let mat: &mut Material = t.downcast_mut().unwrap();
  let mut i = mat.id();
  ui.combo_simple_string("type", &mut i, &["Color", "Texture", "Normal"]);
  if i != mat.id() {
    *mat = Material::default(world, i);
  }
  match mat {
    Material::Color(c) => {
      ui.color_edit3("color", c.as_mut());
    }
    Material::Texture(t) => asset_picker(ui, "texture", world, t),
    Material::Normal => {}
  }
}

fn material_default(world: &mut World) -> Box<dyn Any> {
  Box::new(Material::default(world, 0))
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
            for (i, c) in v.into_iter().enumerate() {
              let id = ui.push_id_usize(i);
              let mut close = true;
              if ui.collapsing_header_with_close_button(
                panel.label,
                TreeNodeFlags::DEFAULT_OPEN,
                &mut close,
              ) {
                hover_tooltip(ui, t.name);
                (panel.render)(c, ui, mutate(world));
              } else {
                hover_tooltip(ui, t.name);
              }
              if !close {
                world.remove_id(t, e);
              }
              id.end();
            }
          }
          None => ui.disabled(true, || {
            ui.collapsing_header(t.name, TreeNodeFlags::empty());
          }),
        }
      }
      ui.separator();
      let [w, _] = ui.window_size();
      if ui.button_with_size("\u{2b} Add Component", [w, 0.0]) {
        ui.open_popup("addcomponent")
      }
      ui.popup("addcomponent", || {
        for (t, i) in panels.iter() {
          if *t != TypeIdNamed::of::<Name>() {
            if ui.selectable_config(i.label).size([w, 0.0]).build() {
              mutate(world)
                .components
                .push_or_insert(*t, (e, (i.default)(mutate(world))));
            }
          }
        }
      });
    }
    None => ui.text("\u{f071} No entity selected."),
  }
}

fn asset_picker<T: Any>(ui: &Ui, label: &str, world: &mut World, handle: &mut Handle<T>) {
  let id = ui.push_id("##");
  let assets = world.get_resource::<Assets>().unwrap();
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
  if let Some(target) = ui.drag_drop_target() {
    if let Some(_) = target.accept_payload_empty(std::any::type_name::<T>(), DragDropFlags::empty())
    {
      let selected = world.get_resource::<SelectedAsset>().unwrap();
      *handle = selected.0.as_ref().unwrap().1.downcast();
    }
  }
  id.end();
}
