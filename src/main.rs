mod gfx;

use glam::{Mat4, Vec3};
use log::LevelFilter;
use anyhow::Result;
use crate::gfx::{Renderer, Shader, Mesh, Texture};

fn main() -> Result<()> {
  env_logger::builder().filter_level(LevelFilter::Info).init();
  let renderer = Renderer::new()?;

  let shader = Shader::new(&renderer.gl, "res/shader.vert", "res/shader.frag")?;
  let mesh = Mesh::load(&renderer.gl, "res/suzanne.obj")?;
  let tex = Texture::new(&renderer.gl, "res/floppa.jpg")?;
  renderer.run(move |context, gl| {
    let size = context.window().inner_size();
    let model = Mat4::from_rotation_y(0.5);
    let view = Mat4::from_translation(Vec3::new(0.0, 0.0, -3.0));
    let projection = Mat4::perspective_rh(0.8, size.width as f32 / size.height as f32, 0.1, 10.0);
    shader.bind(gl);
    shader.set_mat4(gl, 0, model);
    shader.set_mat4(gl, 1, view);
    shader.set_mat4(gl, 2, projection);
    tex.bind(gl);
    mesh.draw(gl);
  })
}
