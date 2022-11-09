use phosphor::Result;
use phosphor::ecs::{World, Name};
use phosphor::gfx::{Texture, Mesh, gl};
use phosphor::math::Vec3;
use phosphor_ui::Textures;
use phosphor_ui::imgui::{Ui, Image, TextureId};
use phosphor_3d::{Camera, Transform, SceneRendererOptions};

pub trait Panel {
  fn setup(_: &mut World) -> Result<Self>
  where
    Self: Sized;
  fn render(&mut self, _: &mut World, _: &Ui) -> Result<()>;
  fn title(&self) -> &'static str;
  fn open(&mut self) -> &mut bool;
}

pub fn setup_panels(world: &mut World) -> Result<()> {
  let scene = Box::new(Scene::setup(world)?);
  let outline = Box::new(Outline::setup(world)?);
  world.add_resource::<Vec<Box<dyn Panel>>>(vec![scene, outline]);
  Ok(())
}

struct Scene {
  open: bool,
  fb: u32,
  tex: TextureId,
}

impl Panel for Scene {
  fn setup(world: &mut World) -> Result<Self> {
    let textures = world.get_resource::<Textures>().unwrap();
    world
      .spawn("cam")
      .insert(
        Transform::new()
          .pos(Vec3::new(0.0, 1.0, -10.0))
          .rot_euler(Vec3::new(0.0, 0.0, 1.5)),
      )
      .insert(Camera::new(0.8, 0.1..100.0));
    world
      .spawn("teapot")
      .insert(Transform::new())
      .insert(Mesh::load("res/teapot.obj")?);
    unsafe {
      let mut fb = 0;
      gl::GenFramebuffers(1, &mut fb);
      gl::BindFramebuffer(gl::FRAMEBUFFER, fb);
      let tex = Texture::new(&[], 0, 0)?;
      gl::FramebufferTexture2D(
        gl::FRAMEBUFFER,
        gl::COLOR_ATTACHMENT0,
        gl::TEXTURE_2D,
        tex.0,
        0,
      );
      let tex = textures.insert(tex);
      Ok(Self {
        open: true,
        fb,
        tex,
      })
    }
  }

  fn render(&mut self, world: &mut World, ui: &Ui) -> Result<()> {
    let size = ui.window_size();
    Image::new(self.tex, size)
      .uv0([0.0, 1.0])
      .uv1([1.0, 0.0])
      .build(&ui);

    let tex = world
      .get_resource::<Textures>()
      .unwrap()
      .get(self.tex)
      .unwrap();
    tex.resize(size[0] as _, size[1] as _);
    world.add_resource(SceneRendererOptions { fb: self.fb, size });
    Ok(())
  }

  fn title(&self) -> &'static str {
    "Scene"
  }

  fn open(&mut self) -> &mut bool {
    &mut self.open
  }
}

struct Outline {
  open: bool,
}

impl Panel for Outline {
  fn setup(_: &mut World) -> Result<Self> {
    Ok(Self { open: true })
  }

  fn render(&mut self, world: &mut World, ui: &Ui) -> Result<()> {
    let [w, _] = ui.window_size();
    for (_, n) in world.query::<Name>() {
      ui.selectable(n.0);
    }
    ui.button_with_size("Add Entity", [w, 0.0]);
    Ok(())
  }

  fn title(&self) -> &'static str {
    "Outline"
  }

  fn open(&mut self) -> &mut bool {
    &mut self.open
  }
}
