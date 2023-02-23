use std::any::Any;
use phosphor::Result;
use phosphor::gfx::{Renderer, Shader, Texture, Mesh, Framebuffer, Vertex, gl};
use phosphor::ecs::{World, stage};
use phosphor::math::{Vec3, Quat, EulerRot, Mat4, Vec2};
use phosphor::assets::{Assets, Handle};
use phosphor::log::{warn, error};
use phosphor::component;
use serde::{Serialize, Deserialize};

const SHADOW_RES: u32 = 4096;

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

  // goofy
  pub fn euler_dir(&self) -> Vec3 {
    euler_dir(self.rotation.x, self.rotation.y)
  }
}

fn euler_dir(yaw: f32, pitch: f32) -> Vec3 {
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
}

#[derive(Serialize, Deserialize)]
#[component]
pub struct Model {
  pub mesh: Handle<Mesh>,
  pub cast_shadows: bool,
}

impl Model {
  pub fn new(mesh: Handle<Mesh>) -> Self {
    Model {
      mesh,
      cast_shadows: true,
    }
  }
}

#[derive(Serialize, Deserialize)]
#[component]
pub enum Material {
  Color(Vec3),
  Texture(Handle<Texture>),
  Normal,
}

impl Material {
  pub fn default(world: &World, id: usize) -> Self {
    match id {
      0 => Self::Color(Vec3::splat(0.75)),
      1 => Self::Texture(
        world
          .get_resource::<Assets>()
          .unwrap()
          .load("brick.jpg")
          .unwrap(),
      ),
      2 => Self::Normal,
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
      Self::Normal => 2,
    }
  }
}

pub struct LightDir(pub Vec2);

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

pub fn scenerenderer(world: &mut World) -> Result<()> {
  world.add_resource(LightDir(Vec2::new(30.0, 300.0)));
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
    sky_shader: Shader::new("assets/sky.vert", "assets/sky.frag")?,
    shadow_fb,
    shadow_tex,
    shadow_shader: Shader::new("assets/shadow.vert", "assets/shadow.frag")?,
    color_shader: Shader::new("assets/base.vert", "assets/color.frag")?,
    texture_shader: Shader::new("assets/base.vert", "assets/texture.frag")?,
    normal_shader: Shader::new("assets/base.vert", "assets/normal.frag")?,
  });
  world.add_system(stage::DRAW, &scenerenderer_draw);
  Ok(())
}

pub struct SceneDrawOptions {
  pub fb: Framebuffer,
  pub size: [f32; 2],
}

fn scenerenderer_draw(world: &mut World) -> Result<()> {
  let renderer = world.get_resource::<Renderer>().unwrap();
  let (w, h) = renderer.window.get_framebuffer_size();
  match world.query::<Camera>().get(0) {
    Some((e, cam)) => match e.get::<Transform>() {
      Some(cam_t) => {
        let r = world.get_resource::<SceneRenderer>().unwrap();
        let light_dir = world.get_resource::<LightDir>().unwrap().0;
        let light_dir = euler_dir(light_dir.x, light_dir.y);

        r.shadow_fb.bind();
        renderer.resize(SHADOW_RES, SHADOW_RES);
        renderer.clear(0.0, 0.0, 0.0, 1.0);
        let light_view = Mat4::look_at_rh(light_dir, Vec3::ZERO, Vec3::Y);
        let light_projection = Mat4::orthographic_rh(-40.0, 40.0, -40.0, 40.0, 0.1, 50.0);
        r.shadow_shader.bind();
        r.shadow_shader.set_mat4("view", &light_view);
        r.shadow_shader.set_mat4("projection", &light_projection);
        for (e, model) in world.query::<Model>() {
          if model.cast_shadows {
            if let Some(model_t) = e.get::<Transform>() {
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

        let view = Mat4::look_to_rh(cam_t.position, cam_t.euler_dir(), Vec3::Y);
        let projection =
          Mat4::perspective_rh(cam.fov.to_radians(), aspect, cam.clip[0], cam.clip[1]);

        r.sky_shader.bind();
        r.sky_shader.set_mat4("view", &view);
        r.sky_shader.set_mat4("projection", &projection);
        r.sky_shader.set_vec3("light_dir", &light_dir);
        unsafe {
          gl::DepthMask(gl::FALSE);
          r.sky_mesh.draw();
          gl::DepthMask(gl::TRUE);
        }

        for (e, model) in world.query::<Model>() {
          match e.get::<Transform>() {
            Some(model_t) => {
              let shader = match e
                .get::<Material>()
                .unwrap_or(&mut Material::default(world, 0))
              {
                Material::Color(col) => {
                  r.color_shader.bind();
                  r.color_shader.set_vec3("color", col);
                  &r.color_shader
                }
                Material::Texture(tex) => {
                  r.texture_shader.bind();
                  tex.bind(gl::TEXTURE0);
                  &r.texture_shader
                }
                Material::Normal => {
                  r.normal_shader.bind();
                  &r.normal_shader
                }
              };

              shader.set_mat4("model", &model_t.as_mat4());
              shader.set_mat4("view", &view);
              shader.set_mat4("projection", &projection);
              shader.set_vec3("light_dir", &light_dir);
              shader.set_mat4("light_view", &light_view);
              shader.set_mat4("light_projection", &light_projection);
              r.shadow_tex.bind(gl::TEXTURE1);
              shader.set_i32("shadow_map", 1);
              model.mesh.draw();
            }
            None => warn!(
              "Mesh on entity {} won't be rendered (Missing Transform).",
              e.id
            ),
          }
        }
      }
      None => warn!("Scene will not be rendered (Missing camera transform)."),
    },
    None => warn!("Scene will not be rendered (Missing camera)."),
  };
  Framebuffer::DEFAULT.bind();
  renderer.resize(w as _, h as _);
  Ok(())
}
