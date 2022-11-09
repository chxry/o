use std::ops::Range;
use phosphor::Result;
use phosphor::gfx::{Renderer, Shader, Texture, Mesh, gl};
use phosphor::ecs::{World, Stage};
use phosphor::math::{Vec3, Quat, EulerRot, Mat4};
use phosphor::log::warn;

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
  pub clip: Range<f32>,
}

impl Camera {
  pub fn new(fov: f32, clip: Range<f32>) -> Self {
    Self { fov, clip }
  }
}

pub enum Material {
  Textured(Texture),
  Color(Vec3),
}

pub struct SceneRendererStage(pub Stage);

pub struct SceneRendererOptions {
  pub fb: u32,
  pub size: [f32; 2],
}

struct SceneRenderer {
  texture_shader: Shader,
  color_shader: Shader,
}

pub fn scenerenderer(world: &mut World) -> Result<()> {
  world.add_resource(SceneRenderer {
    texture_shader: Shader::new("res/base.vert", "res/texture.frag")?,
    color_shader: Shader::new("res/base.vert", "res/color.frag")?,
  });
  world.add_system(
    world
      .get_resource::<SceneRendererStage>()
      .unwrap_or(&mut SceneRendererStage(Stage::Draw))
      .0,
    &scenerenderer_draw,
  );
  Ok(())
}

fn scenerenderer_draw(world: &mut World) -> Result<()> {
  match world.query::<Camera>().get(0) {
    Some((e, cam)) => match e.get::<Transform>() {
      Some(cam_t) => {
        let renderer = world.get_resource::<Renderer>().unwrap();
        let r = world.get_resource::<SceneRenderer>().unwrap();
        let d = SceneRendererOptions {
          fb: 0,
          size: renderer.window.inner_size().into(),
        };
        let opts = match world.get_resource::<SceneRendererOptions>() {
          Some(s) => s,
          None => &d,
        };
        unsafe {
          gl::BindFramebuffer(gl::FRAMEBUFFER, opts.fb);
        }
        renderer.resize(opts.size);
        renderer.clear();

        let view = Mat4::look_to_rh(cam_t.position, cam_t.rotation.to_scaled_axis(), Vec3::Y);
        let projection = Mat4::perspective_rh(
          cam.fov,
          opts.size[0] / opts.size[1],
          cam.clip.start,
          cam.clip.end,
        );

        for (e, mesh) in world.query::<Mesh>() {
          match e.get::<Transform>() {
            Some(mesh_t) => {
              let shader = match e
                .get::<Material>()
                .unwrap_or(&mut Material::Color(Vec3::splat(0.75)))
              {
                Material::Textured(tex) => {
                  r.texture_shader.bind();
                  tex.bind();
                  &r.texture_shader
                }
                Material::Color(col) => {
                  r.color_shader.bind();
                  r.color_shader.set_vec3(3, col);
                  &r.color_shader
                }
              };

              shader.set_mat4(0, &mesh_t.as_mat4());
              shader.set_mat4(1, &view);
              shader.set_mat4(2, &projection);
              mesh.draw();
            }
            None => warn!(
              "Mesh on entity {} won't be rendered (Missing Transform).",
              e.id
            ),
          }
        }
        unsafe {
          gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
      }
      None => warn!("Scene will not be rendered (Missing camera transform)."),
    },
    None => warn!("Scene will not be rendered (Missing camera)."),
  };
  Ok(())
}
