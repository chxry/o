use std::any::Any;
use phosphor::Result;
use phosphor::gfx::{Renderer, Shader, Texture, Mesh, Framebuffer, Vertex, gl};
use phosphor::ecs::{World, stage};
use phosphor::math::{Vec3, Quat, EulerRot, Mat4};
use phosphor::assets::{Assets, Handle};
use phosphor::log::{warn, error};
use phosphor::component;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
#[component]
pub struct Transform {
  pub position: Vec3,
  pub rotation: Vec3,
  pub scale: Vec3,
}

impl Transform {
  pub fn new() -> Self {
    Self {
      position: Vec3::ZERO,
      rotation: Vec3::ZERO,
      scale: Vec3::ONE,
    }
  }

  pub fn pos(mut self, position: Vec3) -> Self {
    self.position = position;
    self
  }

  pub fn rot(mut self, rotation: Vec3) -> Self {
    self.rotation = rotation;
    self
  }

  pub fn scale(mut self, scale: Vec3) -> Self {
    self.scale = scale;
    self
  }

  pub fn as_mat4(&self) -> Mat4 {
    Mat4::from_scale_rotation_translation(
      self.scale,
      Quat::from_euler(
        EulerRot::XYZ,
        self.rotation.x.to_radians(),
        self.rotation.y.to_radians(),
        self.rotation.z.to_radians(),
      ),
      self.position,
    )
  }

  pub fn dir(&self) -> Vec3 {
    Vec3::new(
      self.rotation.y.to_radians().cos() * self.rotation.x.to_radians().cos(),
      self.rotation.x.to_radians().sin(),
      self.rotation.y.to_radians().sin() * self.rotation.x.to_radians().cos(),
    )
  }
}

#[derive(Serialize, Deserialize)]
#[component]
pub struct Camera {
  pub fov: f32,
  pub clip: [f32; 2],
}

impl Camera {
  pub fn new(fov: f32, clip: [f32; 2]) -> Self {
    Self { fov, clip }
  }
}

#[derive(Serialize, Deserialize)]
#[component]
pub struct Model(pub Handle<Mesh>);

#[derive(Serialize, Deserialize)]
#[component]
pub enum Material {
  Color(Vec3),
  Texture(Handle<Texture>),
}

impl Material {
  pub fn default(world: &World, id: usize) -> Self {
    match id {
      0 => Self::Color(Vec3::splat(0.75)),
      1 => Self::Texture(
        world
          .get_resource::<Assets>()
          .unwrap()
          .load("res/brick.jpg")
          .unwrap(),
      ),
      _ => {
        error!("Unknown material {}.", id);
        panic!();
      }
    }
  }

  pub fn id(&self) -> usize {
    match self {
      Self::Color(_) => 0,
      Self::Texture(_) => 1,
    }
  }
}

struct SceneRenderer {
  sky_mesh: Mesh,
  sky_shader: Shader,
  texture_shader: Shader,
  color_shader: Shader,
}

pub fn scenerenderer(world: &mut World) -> Result<()> {
  world.add_resource(SceneRenderer {
    sky_mesh: Mesh::new(
      &[
        Vertex {
          pos: [1.0, 1.0, 0.0],
          uv: [1.0, 1.0],
        },
        Vertex {
          pos: [1.0, -1.0, 0.0],
          uv: [1.0, 0.0],
        },
        Vertex {
          pos: [-1.0, 1.0, 0.0],
          uv: [0.0, 1.0],
        },
        Vertex {
          pos: [-1.0, -1.0, 0.0],
          uv: [0.0, 0.0],
        },
      ],
      &[0, 1, 2, 1, 3, 2],
    ),
    sky_shader: Shader::new("res/sky.vert", "res/sky.frag")?,
    color_shader: Shader::new("res/base.vert", "res/color.frag")?,
    texture_shader: Shader::new("res/base.vert", "res/texture.frag")?,
  });
  world.add_system(stage::DRAW, &scenerenderer_draw);
  Ok(())
}

pub struct SceneDrawOptions {
  pub fb: Framebuffer,
  pub size: [f32; 2],
}

fn scenerenderer_draw(world: &mut World) -> Result<()> {
  match world.query::<Camera>().get(0) {
    Some((e, cam)) => match e.get::<Transform>() {
      Some(cam_t) => {
        let renderer = world.get_resource::<Renderer>().unwrap();
        let r = world.get_resource::<SceneRenderer>().unwrap();
        let (w, h) = renderer.window.get_framebuffer_size();
        let aspect = match world.get_resource::<SceneDrawOptions>() {
          Some(o) => {
            o.fb.bind();
            renderer.resize(o.size[0] as _, o.size[1] as _);
            o.size[0] / o.size[1]
          }
          None => {
            renderer.resize(w as _, h as _);
            w as f32 / h as f32
          }
        };
        renderer.clear(0.0, 0.0, 0.0, 1.0);

        let view = Mat4::look_to_rh(cam_t.position, cam_t.dir(), Vec3::Y);
        let projection =
          Mat4::perspective_rh(cam.fov.to_radians(), aspect, cam.clip[0], cam.clip[1]);

        r.sky_shader.bind();
        r.sky_shader.set_mat4("view", &view);
        r.sky_shader.set_mat4("projection", &projection);
        unsafe {
          gl::DepthMask(gl::FALSE);
          r.sky_mesh.draw();
          gl::DepthMask(gl::TRUE);
        }

        for (e, mesh) in world.query::<Model>() {
          match e.get::<Transform>() {
            Some(mesh_t) => {
              let shader = match e
                .get::<Material>()
                .unwrap_or(&mut Material::default(world, 0))
              {
                Material::Color(col) => {
                  r.color_shader.bind();
                  r.color_shader.set_vec3("u_color", col);
                  &r.color_shader
                }
                Material::Texture(tex) => {
                  r.texture_shader.bind();
                  tex.bind();
                  &r.texture_shader
                }
              };

              shader.set_mat4("model", &mesh_t.as_mat4());
              shader.set_mat4("view", &view);
              shader.set_mat4("projection", &projection);
              mesh.0.draw();
            }
            None => warn!(
              "Mesh on entity {} won't be rendered (Missing Transform).",
              e.id
            ),
          }
        }
        Framebuffer::DEFAULT.bind();
        renderer.resize(w as _, h as _);
      }
      None => warn!("Scene will not be rendered (Missing camera transform)."),
    },
    None => warn!("Scene will not be rendered (Missing camera)."),
  };
  Ok(())
}
