#![feature(vec_into_raw_parts)]
use phosphor::{Result, DeltaTime, component};
use phosphor::ecs::{World, stage};
use phosphor::math::{Vec3, Mat4};
use phosphor::log::debug;
use phosphor::gfx::Mesh;
use phosphor_3d::{Transform, Camera};
use phosphor_imgui::imgui::{Ui, draw_list::DrawListMut};
use rapier3d::prelude::*;
use rapier3d::dynamics::{RigidBody as RapierRigidBody, RigidBodyBuilder as RapierRigidBodyBuilder};
use rapier3d::geometry::{Collider as RapierCollider, ColliderBuilder as RapierColliderBuilder};
use serde::{Serialize, Deserialize};

pub use rapier3d;

pub struct Gravity(pub Vec3);

pub struct RigidBodyBuilder {
  b: RapierRigidBodyBuilder,
}

impl RigidBodyBuilder {
  pub fn fixed() -> Self {
    Self {
      b: RapierRigidBodyBuilder::fixed(),
    }
  }

  pub fn dynamic() -> Self {
    Self {
      b: RapierRigidBodyBuilder::dynamic(),
    }
  }

  pub fn build(self, world: &World) -> RigidBody {
    RigidBody {
      handle: world
        .get_resource::<RigidBodySet>()
        .unwrap()
        .insert(self.b.build()),
    }
  }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
#[component]
pub struct RigidBody {
  pub handle: RigidBodyHandle,
}

impl RigidBody {
  pub fn get<'s>(&'s self, world: &'s World) -> &mut RapierRigidBody {
    world
      .get_resource::<RigidBodySet>()
      .unwrap()
      .get_mut(self.handle)
      .unwrap()
  }
}

pub struct ColliderBuilder {
  b: RapierColliderBuilder,
  rb: Option<RigidBody>,
}

impl ColliderBuilder {
  pub fn ball(r: f32) -> Self {
    ColliderBuilder {
      b: RapierColliderBuilder::ball(r),
      rb: None,
    }
  }

  pub fn cuboid(x: f32, y: f32, z: f32) -> Self {
    ColliderBuilder {
      b: RapierColliderBuilder::cuboid(x, y, z),
      rb: None,
    }
  }

  pub fn convex_hull(mesh: &Mesh) -> Self {
    Self {
      b: RapierColliderBuilder::convex_hull(
        mesh
          .vertices
          .iter()
          .map(|v| v.pos.into())
          .collect::<Vec<_>>()
          .as_slice(),
      )
      .unwrap(),
      rb: None,
    }
  }

  pub fn trimesh(mesh: &Mesh) -> Self {
    let v = mesh.indices.clone().into_raw_parts();
    Self {
      b: RapierColliderBuilder::trimesh(
        mesh
          .vertices
          .iter()
          .map(|v| v.pos.into())
          .collect::<Vec<_>>(),
        unsafe { Vec::from_raw_parts(v.0 as _, v.1 / 3, v.2 / 3) },
      ),
      rb: None,
    }
  }

  pub fn attach_rb(mut self, rb: RigidBody) -> Self {
    self.rb = Some(rb);
    self
  }

  pub fn mass(mut self, mass: f32) -> Self {
    self.b = self.b.mass(mass);
    self
  }

  pub fn build(self, world: &World) -> Collider {
    let set = world.get_resource::<ColliderSet>().unwrap();
    Collider {
      handle: match self.rb {
        Some(rb) => set.insert_with_parent(
          self.b.build(),
          rb.handle,
          world.get_resource::<RigidBodySet>().unwrap(),
        ),
        None => set.insert(self.b.build()),
      },
    }
  }
}

#[derive(Serialize, Deserialize)]
#[component]
pub struct Collider {
  pub handle: ColliderHandle,
}

impl Collider {
  pub fn get<'s>(&'s self, world: &'s World) -> &mut RapierCollider {
    world
      .get_resource::<ColliderSet>()
      .unwrap()
      .get_mut(self.handle)
      .unwrap()
  }
}

pub fn rapier_plugin(world: &mut World) -> Result {
  world.add_resource(PhysicsPipeline::new());
  world.add_resource(Gravity(Vec3::new(0.0, -9.81, 0.0)));
  world.add_resource(IslandManager::new());
  world.add_resource(BroadPhase::new());
  world.add_resource(NarrowPhase::new());
  world.add_resource(RigidBodySet::new());
  world.add_resource(ColliderSet::new());
  world.add_resource(ImpulseJointSet::new());
  world.add_resource(MultibodyJointSet::new());
  world.add_resource(CCDSolver::new());
  world.add_system(stage::PRE_DRAW, rapier_update);
  debug!("Initialized Rapier {}.", rapier3d::VERSION);
  Ok(())
}

pub fn rapier_debug_plugin(world: &mut World) -> Result {
  if world.get_resource::<DebugRenderPipeline>().is_none() {
    world.add_resource(DebugRenderPipeline::render_all(DebugRenderStyle::default()));
  }
  world.add_system(stage::DRAW, debug_update);
  Ok(())
}

fn rapier_update(world: &mut World) -> Result {
  let physics_pipeline = world.get_resource::<PhysicsPipeline>().unwrap();
  let gravity = world.get_resource::<Gravity>().unwrap();
  let island_manager = world.get_resource::<IslandManager>().unwrap();
  let broad_phase = world.get_resource::<BroadPhase>().unwrap();
  let narrow_phase = world.get_resource::<NarrowPhase>().unwrap();
  let rb_set = world.get_resource::<RigidBodySet>().unwrap();
  let collider_set = world.get_resource::<ColliderSet>().unwrap();
  let impulse_joint_set = world.get_resource::<ImpulseJointSet>().unwrap();
  let multibody_joint_set = world.get_resource::<MultibodyJointSet>().unwrap();
  let ccd_solver = world.get_resource::<CCDSolver>().unwrap();
  for (e, collider) in world.query::<Collider>() {
    if e.get_one::<RigidBody>().is_none() {
      if let Some(t) = e.get_one::<Transform>() {
        let collider = collider.get(world);
        collider.set_translation(t.position.into());
        collider.set_rotation(t.rotation.into());
      }
    }
  }
  for (e, rb) in world.query::<RigidBody>() {
    if let Some(t) = e.get_one::<Transform>() {
      let rb = rb.get(world);
      rb.set_translation(t.position.into(), true);
      rb.set_rotation(t.rotation.into(), true);
    }
  }
  physics_pipeline.step(
    &gravity.0.into(),
    &IntegrationParameters {
      dt: world.get_resource::<DeltaTime>().unwrap().0,
      ..Default::default()
    },
    island_manager,
    broad_phase,
    narrow_phase,
    rb_set,
    collider_set,
    impulse_joint_set,
    multibody_joint_set,
    ccd_solver,
    None,
    &(),
    &(),
  );
  for (e, rb) in world.query::<RigidBody>() {
    if let Some(t) = e.get_one::<Transform>() {
      let rb = rb.get(world);
      t.position = (*rb.translation()).into();
      t.rotation = (*rb.rotation()).into();
    }
  }
  Ok(())
}

struct DebugRenderer<'d> {
  ui: DrawListMut<'d>,
  size: [f32; 2],
  view: Mat4,
  proj: Mat4,
}

impl DebugRenderBackend for DebugRenderer<'_> {
  fn draw_line(&mut self, _: DebugRenderObject, a: Point<Real>, b: Point<Real>, color: [f32; 4]) {
    let mut a = self
      .proj
      .project_point3(self.view.transform_point3(a.into()));
    if a.x > 1.0 || a.x < -1.0 || a.y > 1.0 || a.y < -1.0 || a.z > 1.0 {
      return;
    }
    let mut b = self
      .proj
      .project_point3(self.view.transform_point3(b.into()));
    if b.x > 1.0 || b.x < -1.0 || b.y > 1.0 || b.y < -1.0 || b.z > 1.0 {
      return;
    }
    a.x = (a.x / 2.0 + 0.5) * self.size[0];
    a.y = (a.y / -2.0 + 0.5) * self.size[1];
    b.x = (b.x / 2.0 + 0.5) * self.size[0];
    b.y = (b.y / -2.0 + 0.5) * self.size[1];
    self.ui.add_line([a.x, a.y], [b.x, b.y], color).build();
  }
}

fn debug_update(world: &mut World) -> Result {
  let debug_pipeline = world.get_resource::<DebugRenderPipeline>().unwrap();
  let rb_set = world.get_resource::<RigidBodySet>().unwrap();
  let collider_set = world.get_resource::<ColliderSet>().unwrap();
  let impulse_joint_set = world.get_resource::<ImpulseJointSet>().unwrap();
  let multibody_joint_set = world.get_resource::<MultibodyJointSet>().unwrap();
  let narrow_phase = world.get_resource::<NarrowPhase>().unwrap();
  let ui = world.get_resource::<Ui>().unwrap();
  let size = ui.io().display_size;
  let (e, cam) = &world.query::<Camera>()[0];
  let cam_t = e.get_one::<Transform>().unwrap();
  let (view, proj) = cam.matrices(cam_t, size[0] / size[1]);
  debug_pipeline.render(
    &mut DebugRenderer {
      ui: ui.get_background_draw_list(),
      size,
      view,
      proj,
    },
    rb_set,
    collider_set,
    impulse_joint_set,
    multibody_joint_set,
    narrow_phase,
  );
  Ok(())
}
