use std::collections::HashMap;
use std::any::Any;
use phosphor::ecs::{World, Name, TypeIdNamed};
use phosphor_ui::hover_tooltip;
use phosphor_ui::imgui::{Ui, Drag, WindowFlags, TreeNodeFlags};
use phosphor_3d::{Camera, Transform};
use crate::SelectedEntity;
use crate::panels::Panel;

pub fn init(world: &mut World) -> Panel {
  let mut panels = HashMap::new();
  panels.insert(
    TypeIdNamed::of::<Name>(),
    InspectorPanel {
      label: "\u{e261} Name",
      render: &inspector_name,
    },
  );
  panels.insert(
    TypeIdNamed::of::<Transform>(),
    InspectorPanel {
      label: "\u{e428} Transform",
      render: &inspector_transform,
    },
  );
  panels.insert(
    TypeIdNamed::of::<Camera>(),
    InspectorPanel {
      label: "\u{e3b0} Camera",
      render: &inspector_camera,
    },
  );
  world.add_resource(panels);
  Panel {
    title: "\u{e88e} Inspector",
    flags: WindowFlags::empty(),
    vars: &[],
    open: true,
    render: &render,
  }
}

struct InspectorPanel {
  pub label: &'static str,
  pub render: &'static dyn Fn(&mut Box<dyn Any>, &Ui),
}

fn inspector_name(t: &mut Box<dyn Any>, ui: &Ui) {
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
fn inspector_transform(t: &mut Box<dyn Any>, ui: &Ui) {
  let transform: &mut Transform = t.downcast_mut().unwrap();
  Drag::new("pos")
    .speed(0.05)
    .display_format("%.5g")
    .build_array(ui, transform.position.as_mut());
  Drag::new("scale")
    .speed(0.05)
    .display_format("%.5g")
    .build_array(ui, transform.scale.as_mut());
}

fn inspector_camera(t: &mut Box<dyn Any>, ui: &Ui) {
  let cam: &mut Camera = t.downcast_mut().unwrap();
  Drag::new("fov")
    .speed(0.05)
    .display_format("%.5g")
    .build(ui, &mut cam.fov);
  Drag::new("clip")
    .speed(0.05)
    .display_format("%.5g")
    .build_array(ui, &mut [cam.clip.start, cam.clip.end]);
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
                (panel.render)(c, ui);
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
      let [w, _] = ui.window_size();
      ui.button_with_size("\u{e145} Add Component", [w, 0.0]);
    }
    None => ui.text("\u{e002} No entity selected."),
  }
}
