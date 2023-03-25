#![allow(clippy::new_without_default)]
use phosphor::Result;
use phosphor::gfx::{Renderer, Shader, Texture, Mesh, Framebuffer, Vertex, gl};
use phosphor::ecs::{World, Name, stage};
use phosphor::math::{Vec3, Quat, Mat4, Vec2, EulerRot};
use phosphor::assets::{Assets, Handle};
use phosphor::log::error;
use phosphor::component;
use log_once::warn_once;
use serde::{Serialize, Deserialize};

const SHADOW_RES: u32 = 4096;

#[derive(Serialize, Deserialize)]
#[component]
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

  pub fn rot(mut self, rotation: Quat) -> Self {
    self.rotation = rotation;
    self
  }

  pub fn rot_euler(mut self, y: f32, p: f32, r: f32) -> Self {
    self.rotation = Quat::from_euler(
      EulerRot::YXZ,
      y.to_radians(),
      p.to_radians(),
      r.to_radians(),
    );
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

fn dir(yaw: f32, pitch: f32) -> Vec3 {
  Vec3::new(
    pitch.to_radians().cos() * yaw.to_radians().cos(),
    yaw.to_radians().sin(),
    pitch.to_radians().sin() * yaw.to_radians().cos(),
  )
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

  pub fn matrices(&self, transform: &Transform, aspect: f32) -> (Mat4, Mat4) {
    (
      Mat4::look_to_rh(
        transform.position,
        transform.rotation * Vec3::NEG_Z,
        Vec3::Y,
      ),
      Mat4::perspective_rh(self.fov.to_radians(), aspect, self.clip[0], self.clip[1]),
    )
  }
}

#[derive(Serialize, Deserialize)]
#[component]
pub struct Model {
  pub mesh: Handle<Mesh>,
  pub cast_shadows: bool,
  pub wireframe: bool,
}

impl Model {
  pub fn new(mesh: Handle<Mesh>) -> Self {
    Self {
      mesh,
      cast_shadows: true,
      wireframe: false,
    }
  }
}

#[derive(Serialize, Deserialize)]
#[component]
pub enum Material {
  Color { color: Vec3, spec: f32 },
  Texture { tex: Handle<Texture>, spec: f32 },
  Normal,
}

impl Material {
  pub fn default(world: &World, id: usize) -> Self {
    match id {
      0 => Self::Color {
        color: Vec3::splat(0.75),
        spec: 0.5,
      },
      1 => Self::Texture {
        tex: world
          .get_resource::<Assets>()
          .unwrap()
          .load("garfield.png")
          .unwrap(),
        spec: 0.5,
      },
      2 => Self::Normal,
      _ => {
        error!("Unknown material {}.", id);
        panic!();
      }
    }
  }

  pub fn id(&self) -> usize {
    match self {
      Self::Color { .. } => 0,
      Self::Texture { .. } => 1,
      Self::Normal => 2,
    }
  }
}

#[derive(Serialize, Deserialize)]
#[component]
pub struct Light {
  pub color: Vec3,
  pub strength: f32,
}

impl Light {
  pub fn new(color: Vec3) -> Self {
    Self {
      color,
      strength: 2.5,
    }
  }

  pub fn strength(mut self, strength: f32) -> Self {
    self.strength = strength;
    self
  }
}

pub struct SkySettings {
  pub dir: Vec2,
}

struct SceneRenderer {
  sky_mesh: Mesh,
  sky_shader: Shader,
  shadow_fb: Framebuffer,
  shadow_tex: Texture,
  shadow_shader: Shader,
  color_shader: Shader,
  texture_shader: Shader,
  normal_shader: Shader,
}

pub fn scenerenderer_plugin(world: &mut World) -> Result {
  world.add_resource(SkySettings {
    dir: Vec2::new(30.0, 300.0),
  });
  let mut shadow_fb = Framebuffer { fb: 0, rb: 0 };
  let mut shadow_tex = Texture {
    id: 0,
    width: SHADOW_RES,
    height: SHADOW_RES,
  };
  unsafe {
    gl::GenFramebuffers(1, &mut shadow_fb.fb);
    gl::GenTextures(1, &mut shadow_tex.id);
    gl::BindTexture(gl::TEXTURE_2D, shadow_tex.id);
    gl::TexImage2D(
      gl::TEXTURE_2D,
      0,
      gl::DEPTH_COMPONENT as _,
      SHADOW_RES as _,
      SHADOW_RES as _,
      0,
      gl::DEPTH_COMPONENT,
      gl::FLOAT,
      0 as _,
    );
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);
    gl::BindFramebuffer(gl::FRAMEBUFFER, shadow_fb.fb);
    gl::FramebufferTexture2D(
      gl::FRAMEBUFFER,
      gl::DEPTH_ATTACHMENT,
      gl::TEXTURE_2D,
      shadow_tex.id,
      0,
    );
  }
  world.add_resource(SceneRenderer {
    sky_mesh: Mesh::new(
      &[
        Vertex {
          pos: [1.0, 1.0, 0.0],
          uv: [1.0, 1.0],
          normal: [0.0, 0.0, 0.0],
        },
        Vertex {
          pos: [1.0, -1.0, 0.0],
          uv: [1.0, 0.0],
          normal: [0.0, 0.0, 0.0],
        },
        Vertex {
          pos: [-1.0, 1.0, 0.0],
          uv: [0.0, 1.0],
          normal: [0.0, 0.0, 0.0],
        },
        Vertex {
          pos: [-1.0, -1.0, 0.0],
          uv: [0.0, 0.0],
          normal: [0.0, 0.0, 0.0],
        },
      ],
      &[0, 1, 2, 1, 3, 2],
    ),
    sky_shader: Shader::new("sky.vert", "sky.frag")?,
    shadow_fb,
    shadow_tex,
    shadow_shader: Shader::new("shadow.vert", "shadow.frag")?,
    color_shader: Shader::new("base.vert", "color.frag")?,
    texture_shader: Shader::new("base.vert", "texture.frag")?,
    normal_shader: Shader::new("base.vert", "normal.frag")?,
  });
  world.add_system(stage::DRAW, scenerenderer_draw);
  Ok(())
}

pub struct SceneDrawOptions {
  pub fb: Framebuffer,
  pub size: [f32; 2],
}

fn scenerenderer_draw(world: &mut World) -> Result {
  let renderer = world.get_resource::<Renderer>().unwrap();
  let (w, h) = renderer.window.get_framebuffer_size();
  match world.query::<Camera>().get(0) {
    Some((e, cam)) => match e.get_one::<Transform>() {
      Some(cam_t) => {
        let r = world.get_resource::<SceneRenderer>().unwrap();
        let sky = world.get_resource::<SkySettings>().unwrap();
        let sun_dir = dir(sky.dir.x, sky.dir.y);

        r.shadow_fb.bind();
        renderer.resize(SHADOW_RES, SHADOW_RES);
        renderer.clear(0.0, 0.0, 0.0, 1.0);
        let sun_view = Mat4::look_at_rh(sun_dir, Vec3::ZERO, Vec3::Y);
        let sun_projection = Mat4::orthographic_rh(-40.0, 40.0, -40.0, 40.0, 0.1, 50.0);
        r.shadow_shader.bind();
        r.shadow_shader.set_mat4("view", &sun_view);
        r.shadow_shader.set_mat4("projection", &sun_projection);
        for (e, model) in world.query::<Model>() {
          if model.cast_shadows {
            if let Some(model_t) = e.get_one::<Transform>() {
              r.shadow_shader.set_mat4("model", &model_t.as_mat4());
              model.mesh.draw();
            }
          }
        }

        let aspect = match world.get_resource::<SceneDrawOptions>() {
          Some(o) => {
            o.fb.bind();
            renderer.resize(o.size[0] as _, o.size[1] as _);
            o.size[0] / o.size[1]
          }
          None => {
            Framebuffer::DEFAULT.bind();
            renderer.resize(w as _, h as _);
            w as f32 / h as f32
          }
        };
        renderer.clear(0.0, 0.0, 0.0, 1.0);

        let (view, projection) = cam.matrices(cam_t, aspect);

        r.sky_shader.bind();
        r.sky_shader.set_mat4("view", &view);
        r.sky_shader.set_mat4("projection", &projection);
        r.sky_shader.set_vec3("sun_dir", &sun_dir);
        unsafe {
          gl::DepthMask(gl::FALSE);
          r.sky_mesh.draw();
          gl::DepthMask(gl::TRUE);
        }

        for s in [r.color_shader, r.texture_shader, r.normal_shader] {
          s.bind();
          s.set_mat4("view", &view);
          s.set_mat4("projection", &projection);
          s.set_vec3("cam_pos", &cam_t.position);
          s.set_vec3("sun_dir", &sun_dir);
          s.set_mat4("sun_view", &sun_view);
          s.set_mat4("sun_projection", &sun_projection);
          let lights = world.query::<Light>();
          for (i, (e, light)) in lights.iter().enumerate() {
            match e.get_one::<Transform>() {
              Some(light_t) => {
                s.set_vec3(&format!("lights[{}].pos", i), &light_t.position);
                s.set_vec3(&format!("lights[{}].color", i), &light.color);
                s.set_f32(&format!("lights[{}].strength", i), light.strength);
              }
              None => warn_once!(
                "Light on entity '{}'({}) will not be rendered (Missing transform).",
                e.get_one::<Name>().map_or("?", |n| &n.0),
                e.id
              ),
            }
          }
          s.set_i32("num_lights", lights.len() as _);
          s.set_i32("shadow_map", 1);
        }

        r.shadow_tex.bind(1);
        for (e, model) in world.query::<Model>() {
          match e.get_one::<Transform>() {
            Some(model_t) => {
              let shader = match e
                .get_one::<Material>()
                .unwrap_or(&mut Material::default(world, 0))
              {
                Material::Color { color, spec } => {
                  r.color_shader.bind();
                  r.color_shader.set_vec3("color", color);
                  r.color_shader.set_f32("specular", *spec);
                  &r.color_shader
                }
                Material::Texture { tex, spec } => {
                  r.texture_shader.bind();
                  tex.bind(0);
                  r.texture_shader.set_f32("specular", *spec);
                  &r.texture_shader
                }
                Material::Normal => {
                  r.normal_shader.bind();
                  &r.normal_shader
                }
              };
              shader.set_mat4("model", &model_t.as_mat4());
              unsafe {
                gl::PolygonMode(
                  gl::FRONT_AND_BACK,
                  if model.wireframe { gl::LINE } else { gl::FILL },
                );
              }
              model.mesh.draw();
            }
            None => warn_once!(
              "Mesh on entity '{}'({}) won't be rendered (Missing Transform).",
              e.get_one::<Name>().map_or("?", |n| &n.0),
              e.id
            ),
          }
        }
      }
      None => warn_once!("Scene will not be rendered (Missing camera transform)."),
    },
    None => warn_once!("Scene will not be rendered (Missing camera)."),
  };
  unsafe {
    gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
  }
  Framebuffer::DEFAULT.bind();
  renderer.resize(w as _, h as _);
  Ok(())
}
