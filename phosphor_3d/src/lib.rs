use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::any::Any;
use phosphor::Result;
use phosphor::gfx::{Renderer, Shader, Texture, Mesh, Framebuffer, Vertex, gl};
use phosphor::ecs::{World, Stage};
use phosphor::math::{Vec3, Quat, EulerRot, Mat4};
use phosphor::assets::Handle;
use phosphor::log::warn;

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

pub struct Camera {
  pub fov: f32,
  pub clip: [f32; 2],
}

impl Camera {
  pub fn new(fov: f32, clip: [f32; 2]) -> Self {
    Self { fov, clip }
  }
}

pub struct Material {
  pub id: usize,
  pub data: Box<dyn Any>,
}

impl Hash for Material {
  fn hash<H: Hasher>(&self, h: &mut H) {
    self.id.hash(h)
  }
}

impl PartialEq for Material {
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
  }
}

impl Eq for Material {}

impl Material {
  pub fn color(col: Vec3) -> Self {
    Self {
      id: 0,
      data: Box::new(col),
    }
  }

  pub fn texture(tex: Handle<Texture>) -> Self {
    Self {
      id: 1,
      data: Box::new(tex),
    }
  }
}

struct MaterialShader {
  shader: Shader,
  bind: &'static dyn Fn(&Shader, &Box<dyn Any>),
}

fn material_color(shader: &Shader, data: &Box<dyn Any>) {
  let col: &Vec3 = data.downcast_ref().unwrap();
  shader.set_vec3("u_color", col);
}

fn material_texture(_: &Shader, data: &Box<dyn Any>) {
  let tex: &Handle<Texture> = data.downcast_ref().unwrap();
  tex.bind();
}

struct Sky {
  mesh: Mesh,
  shader: Shader,
}

pub fn scenerenderer(world: &mut World) -> Result<()> {
  world.add_resource(HashMap::from([
    (
      0usize,
      MaterialShader {
        shader: Shader::new("res/base.vert", "res/color.frag")?,
        bind: &material_color,
      },
    ),
    (
      1,
      MaterialShader {
        shader: Shader::new("res/base.vert", "res/texture.frag")?,
        bind: &material_texture,
      },
    ),
  ]));
  world.add_resource(Sky {
    mesh: Mesh::new(
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
    shader: Shader::new("res/sky.vert", "res/sky.frag")?,
  });
  world.add_system(Stage::Draw, &scenerenderer_draw);
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
        let mats = world
          .get_resource::<HashMap<usize, MaterialShader>>()
          .unwrap();

        let view = Mat4::look_to_rh(cam_t.position, cam_t.dir(), Vec3::Y);
        let projection =
          Mat4::perspective_rh(cam.fov.to_radians(), aspect, cam.clip[0], cam.clip[1]);

        let sky = world.get_resource::<Sky>().unwrap();
        sky.shader.bind();
        sky.shader.set_mat4("view", &view);
        sky.shader.set_mat4("projection", &projection);
        unsafe {
          gl::DepthMask(gl::FALSE);
          sky.mesh.draw();
          gl::DepthMask(gl::TRUE);
        }

        for (e, mesh) in world.query::<Handle<Mesh>>() {
          match e.get::<Transform>() {
            Some(mesh_t) => {
              let d = &mut Material::color(Vec3::X);
              let mat = e.get::<Material>().unwrap_or(d);
              let s = mats.get(&mat.id).unwrap();
              s.shader.bind();
              s.shader.set_mat4("model", &mesh_t.as_mat4());
              s.shader.set_mat4("view", &view);
              s.shader.set_mat4("projection", &projection);
              (s.bind)(&s.shader, &mat.data);
              mesh.draw();
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
