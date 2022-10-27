use glam::{Vec3, Quat, EulerRot, Mat4};
use log::warn;
use anyhow::Result;
use crate::gfx::{Shader, Texture, Mesh};
use crate::ecs::{Context, Stage};

pub struct Transform {
  pub position: Vec3,
  pub rotation: Quat,
  pub scale: Vec3,
}

impl Transform {
  pub fn new() -> Self {
    Self {
      position: Vec3::ZERO,
      rotation: Quat::IDENTITY,
      scale: Vec3::ONE,
    }
  }

  pub fn pos(mut self, position: Vec3) -> Self {
    self.position = position;
    self
  }

  pub fn rot_quat(mut self, rotation: Quat) -> Self {
    self.rotation = rotation;
    self
  }

  pub fn rot_euler(mut self, rotation: Vec3) -> Self {
    self.rotation = Quat::from_euler(EulerRot::XYZ, rotation.x, rotation.y, rotation.z);
    self
  }

  pub fn scale(mut self, scale: Vec3) -> Self {
    self.scale = scale;
    self
  }

  pub fn as_mat4(&self) -> Mat4 {
    Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
  }
}

pub struct Camera {
  pub fov: f32,
}

impl Camera {
  pub fn new(fov: f32) -> Self {
    Self { fov }
  }
}

struct SceneRenderer {
  shader: Shader,
  tex: Texture,
}

pub fn scene_renderer_init(ctx: Context) -> Result<()> {
  ctx.world.add_resource(SceneRenderer {
    shader: Shader::new(ctx.renderer, "res/shader.vert", "res/shader.frag")?,
    tex: Texture::new(ctx.renderer, "res/floppa.jpg")?,
  });
  ctx.world.add_system(Stage::Draw, &scene_renderer_draw);
  Ok(())
}

fn scene_renderer_draw(ctx: Context) -> Result<()> {
  match ctx.world.query::<Camera>().get(0) {
    Some((e, cam)) => match ctx.world.get::<Transform>(e) {
      Some(cam_t) => {
        let size = ctx.renderer.context.window().inner_size();
        let r = ctx.world.get_resource::<SceneRenderer>().unwrap();
        r.shader.bind(&ctx.renderer);
        r.tex.bind(&ctx.renderer);
        let view = Mat4::look_to_rh(cam_t.position, cam_t.rotation.to_scaled_axis(), Vec3::Y);
        let projection =
          Mat4::perspective_rh(cam.fov, size.width as f32 / size.height as f32, 0.1, 10.0);
        r.shader.bind(&ctx.renderer);
        r.shader.set_mat4(&ctx.renderer, 1, view);
        r.shader.set_mat4(&ctx.renderer, 2, projection);
        for (e, mesh) in ctx.world.query::<Mesh>() {
          match ctx.world.get::<Transform>(e) {
            Some(t) => {
              r.shader.set_mat4(&ctx.renderer, 0, t.as_mat4());
              mesh.draw(&ctx.renderer);
            }
            None => warn!("Mesh on {:?} won't be rendered (Missing Transform).", e),
          }
        }
      }
      None => warn!("Scene will not be rendered (Missing camera transform)."),
    },
    None => warn!("Scene will not be rendered (Missing camera)."),
  };
  Ok(())
}
