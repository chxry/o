#![allow(clippy::new_without_default)]
use std::ptr;
use phosphor::Result;
use phosphor::gfx::{Renderer, Shader, Texture, Mesh, Framebuffer, Vertex, Query, gl};
use phosphor::ecs::{World, Name, stage};
use phosphor::math::{Vec3, Quat, Mat4, Vec2, EulerRot};
use phosphor::assets::Handle;
use phosphor::component;
use log_once::warn_once;
use rand::Rng;
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
pub struct Material {
  pub color: Vec3,
  pub tex: Option<Handle<Texture>>,
  pub spec: f32,
  pub metallic: f32,
}

impl Material {
  pub const DEFAULT: Self = Self {
    color: Vec3::splat(0.8),
    tex: None,
    spec: 0.5,
    metallic: 0.0,
  };
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
  gbuffer: Framebuffer,
  galbedo: Texture,
  gposition: Texture,
  gnormal: Texture,
  gmaterial: Texture,
  quad: Mesh,
  light_shader: Shader,
  ssao_samples: Vec<Vec3>,
  ssao_noise: Texture,
  ssao_fb: Framebuffer,
  ssao_tex: Texture,
  ssao_shader: Shader,
  sky_mesh: Mesh,
  sky_shader: Shader,
  shadow_fb: Framebuffer,
  shadow_tex: Texture,
  shadow_shader: Shader,
  default_shader: Shader,
}

pub struct ScenePerf {
  pub shadow_pass: Query,
  pub geometry_pass: Query,
  pub ssao_pass: Query,
  pub lighting_pass: Query,
}

#[derive(Copy, Clone)]
pub enum Tonemap {
  Aces,
  Filmic,
  Reinhard,
  Uncharted2,
}

impl Tonemap {
  pub const ALL: [Self; 4] = [Self::Aces, Self::Filmic, Self::Reinhard, Self::Uncharted2];

  pub fn name(&self) -> &str {
    match self {
      Self::Aces => "Aces",
      Self::Filmic => "Filmic",
      Self::Reinhard => "Reinhard",
      Self::Uncharted2 => "Uncharted2",
    }
  }
}

fn gbuf() -> Texture {
  Texture::new(ptr::null(), 0, 0, gl::RGBA16F, gl::RGBA, gl::FLOAT)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
  a + t * (b - a)
}

pub fn scenerenderer_plugin(world: &mut World) -> Result {
  world.add_resource(SkySettings {
    dir: Vec2::new(30.0, 320.0),
  });
  let gbuffer = Framebuffer::new();
  let galbedo = gbuf();
  gbuffer.bind_tex(&galbedo, 0);
  let gposition = gbuf();
  gbuffer.bind_tex(&gposition, 1);
  let gnormal = gbuf();
  gbuffer.bind_tex(&gnormal, 2);
  let gmaterial = gbuf();
  gbuffer.bind_tex(&gmaterial, 3);
  unsafe {
    gl::DrawBuffers(
      4,
      [
        gl::COLOR_ATTACHMENT0,
        gl::COLOR_ATTACHMENT1,
        gl::COLOR_ATTACHMENT2,
        gl::COLOR_ATTACHMENT3,
      ]
      .as_ptr(),
    );
  }

  let mut rng = rand::thread_rng();
  let mut ssao_samples = vec![];
  for i in 0..64 {
    ssao_samples.push(
      Vec3::new(
        rng.gen_range(-1.0..1.0),
        rng.gen_range(-1.0..1.0),
        rng.gen_range(0.0..1.0),
      )
      .normalize()
        * rng.gen_range(0.0..1.0)
        * lerp(0.0, 1.0, (i as f32 / 64.0).powi(2)),
    );
  }
  let mut ssao_noise = vec![];
  for _ in 0..16 {
    ssao_noise.push(Vec3::new(
      rng.gen_range(-1.0..1.0),
      rng.gen_range(-1.0..1.0),
      0.0,
    ));
  }
  let ssao_noise = Texture::new(
    ssao_noise.as_ptr() as _,
    4,
    4,
    gl::RGBA16F,
    gl::RGB,
    gl::FLOAT,
  );
  let ssao_fb = Framebuffer::new_no_depth();
  let ssao_tex = Texture::new(ptr::null(), 0, 0, gl::RED, gl::RED, gl::FLOAT);
  ssao_fb.bind_tex(&ssao_tex, 0);

  let shadow_fb = Framebuffer::new_no_depth();
  let shadow_tex = Texture::new(
    ptr::null(),
    SHADOW_RES,
    SHADOW_RES,
    gl::DEPTH_COMPONENT,
    gl::DEPTH_COMPONENT,
    gl::FLOAT,
  );
  shadow_fb.bind_depth(&shadow_tex);
  world.add_resource(SceneRenderer {
    gbuffer,
    galbedo,
    gposition,
    gnormal,
    gmaterial,
    quad: Mesh::new(
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
          pos: [-1.0, -1.0, 0.0],
          uv: [0.0, 0.0],
          normal: [0.0, 0.0, 0.0],
        },
        Vertex {
          pos: [-1.0, 1.0, 0.0],
          uv: [0.0, 1.0],
          normal: [0.0, 0.0, 0.0],
        },
      ],
      &[0, 1, 3, 1, 2, 3],
    ),
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
    light_shader: Shader::new("light.vert", "light.frag")?,
    ssao_samples,
    ssao_noise,
    ssao_fb,
    ssao_tex,
    ssao_shader: Shader::new("light.vert", "ssao.frag")?,
    sky_shader: Shader::new("sky.vert", "sky.frag")?,
    shadow_fb,
    shadow_tex,
    shadow_shader: Shader::new("shadow.vert", "shadow.frag")?,
    default_shader: Shader::new("base.vert", "default.frag")?,
  });
  world.add_resource(ScenePerf {
    shadow_pass: Query::new(),
    geometry_pass: Query::new(),
    ssao_pass: Query::new(),
    lighting_pass: Query::new(),
  });
  world.add_resource(Tonemap::Reinhard);
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
        let perf = world.get_resource::<ScenePerf>().unwrap();
        let sky = world.get_resource::<SkySettings>().unwrap();
        let sun_dir = dir(sky.dir.x, sky.dir.y);
        let sun_view = Mat4::look_at_rh(sun_dir, Vec3::ZERO, Vec3::Y);
        // todo calculate this from cam frustum
        let sun_projection = Mat4::orthographic_rh(-15.0, 15.0, -15.0, 15.0, 0.1, 15.0);

        // shadow pass
        perf.shadow_pass.time(|| {
          r.shadow_fb.bind();
          renderer.resize(SHADOW_RES, SHADOW_RES);
          renderer.clear(0.0, 0.0, 0.0, 1.0);
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
        });

        let (fb, w, h) = match world.get_resource::<SceneDrawOptions>() {
          Some(o) => (o.fb, o.size[0], o.size[1]),
          None => (Framebuffer::DEFAULT, w as _, h as _),
        };
        let (view, projection) = cam.matrices(cam_t, w / h);
        // geometry pass
        perf.geometry_pass.time(|| {
          r.gbuffer.bind();
          renderer.resize(w as _, h as _);
          r.gbuffer.resize(w as _, h as _);
          r.galbedo.resize(w as _, h as _);
          r.gposition.resize(w as _, h as _);
          r.gnormal.resize(w as _, h as _);
          r.gmaterial.resize(w as _, h as _);
          r.ssao_fb.resize(w as _, h as _);
          r.ssao_tex.resize(w as _, h as _);
          renderer.clear(0.0, 0.0, 0.0, 1.0);

          r.sky_shader.bind();
          r.sky_shader.set_mat4("view", &view);
          r.sky_shader.set_mat4("projection", &projection);
          r.sky_shader.set_vec3("sun_dir", &sun_dir);
          unsafe {
            gl::DepthMask(gl::FALSE);
            r.sky_mesh.draw();
            gl::DepthMask(gl::TRUE);
          }

          r.default_shader.bind();
          r.default_shader.set_mat4("view", &view);
          r.default_shader.set_mat4("projection", &projection);
          for (e, model) in world.query::<Model>() {
            match e.get_one::<Transform>() {
              Some(model_t) => {
                let mat = match e.get_one::<Material>() {
                  Some(m) => m,
                  None => &Material::DEFAULT,
                };
                match &mat.tex {
                  Some(tex) => {
                    tex.bind(0);
                    r.default_shader.set_i32("use_tex", &1);
                  }
                  None => r.default_shader.set_i32("use_tex", &0),
                };
                r.default_shader.set_vec3("color", &mat.color);
                r.default_shader.set_f32("spec", &mat.spec);
                r.default_shader.set_f32("metallic", &mat.metallic);
                r.default_shader.set_mat4("model", &model_t.as_mat4());
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
        });

        // ssao pass
        perf.ssao_pass.time(|| {
          r.ssao_fb.bind();
          renderer.clear(0.0, 0.0, 0.0, 1.0);
          r.ssao_shader.bind();
          r.galbedo.bind(0);
          r.ssao_shader.set_i32("galbedo", &0);
          r.gposition.bind(1);
          r.ssao_shader.set_i32("gposition", &1);
          r.gnormal.bind(2);
          r.ssao_shader.set_i32("gnormal", &2);
          r.ssao_noise.bind(3);
          r.ssao_shader.set_i32("noise", &3);
          for (i, s) in r.ssao_samples.iter().enumerate() {
            r.ssao_shader.set_vec3(&format!("samples[{}]", i), s);
          }
          r.ssao_shader.set_mat4("view", &view);
          r.ssao_shader.set_mat4("projection", &projection);
          r.quad.draw();
        });

        // lighting pass
        perf.lighting_pass.time(|| {
          fb.bind();
          renderer.clear(0.0, 0.0, 0.0, 1.0);
          r.light_shader.bind();
          r.galbedo.bind(0);
          r.light_shader.set_i32("galbedo", &0);
          r.gposition.bind(1);
          r.light_shader.set_i32("gposition", &1);
          r.gnormal.bind(2);
          r.light_shader.set_i32("gnormal", &2);
          r.gmaterial.bind(3);
          r.light_shader.set_i32("gmaterial", &3);
          r.ssao_tex.bind(4);
          r.light_shader.set_i32("ssao_tex", &4);
          r.shadow_tex.bind(5);
          r.light_shader.set_mat4("view", &view);
          r.light_shader.set_mat4("projection", &projection);
          r.light_shader.set_i32("shadow_map", &5);
          r.light_shader.set_vec3("cam_pos", &cam_t.position);
          r.light_shader.set_vec3("sun_dir", &sun_dir);
          r.light_shader.set_mat4("sun_view", &sun_view);
          r.light_shader.set_mat4("sun_projection", &sun_projection);
          r.light_shader.set_i32(
            "tonemap",
            &(*world
              .get_resource::<Tonemap>()
              .unwrap_or(&mut Tonemap::Aces) as i32),
          );
          let lights = world.query::<Light>();
          for (i, (e, light)) in lights.iter().enumerate() {
            match e.get_one::<Transform>() {
              Some(light_t) => {
                r.light_shader
                  .set_vec3(&format!("lights[{}].pos", i), &light_t.position);
                r.light_shader
                  .set_vec3(&format!("lights[{}].color", i), &light.color);
                r.light_shader
                  .set_f32(&format!("lights[{}].strength", i), &light.strength);
              }
              None => warn_once!(
                "Light on entity '{}'({}) will not be rendered (Missing transform).",
                e.get_one::<Name>().map_or("?", |n| &n.0),
                e.id
              ),
            }
          }
          r.light_shader.set_i32("num_lights", &(lights.len() as _));
          r.quad.draw();
        });
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
