use phosphor::{Result, grr, mutate};
use phosphor::ecs::{World, Name};
use phosphor::gfx::{Texture, Renderer, Mesh};
use phosphor::math::Vec3;
use phosphor_ui::Textures;
use phosphor_ui::imgui::{Ui, Image, TextureId};
use phosphor_3d::{scenerenderer_draw, Camera, Transform, SceneAspect};

pub trait Panel {
  fn setup(_: &mut World) -> Result<Self>
  where
    Self: Sized;
  fn update(&mut self, _: &mut World) -> Result<()> {
    Ok(())
  }
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

pub fn update_panels(world: &mut World) -> Result<()> {
  for panel in world.get_resource::<Vec<Box<dyn Panel>>>().unwrap() {
    panel.update(mutate(world))?;
  }
  Ok(())
}

struct Scene {
  open: bool,
  fb: grr::Framebuffer,
  tex: TextureId,
  size: [f32; 2],
}

impl Panel for Scene {
  fn setup(world: &mut World) -> Result<Self> {
    let renderer = world.get_resource::<Renderer>().unwrap();
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
      .insert(Mesh::load(renderer, "res/teapot.obj")?);
    unsafe {
      let fb = renderer.gl.create_framebuffer()?;
      let tex = Texture::empty(renderer, 1920, 1080)?; // todo resize
      renderer.gl.bind_attachments(
        fb,
        &[(
          grr::Attachment::Color(0),
          grr::AttachmentView::Image(tex.view()),
        )],
      );
      Ok(Self {
        open: true,
        fb,
        tex: textures.insert(tex),
        size: [1920.0, 1080.0],
      })
    }
  }

  fn update(&mut self, world: &mut World) -> Result<()> {
    if self.open {
      let renderer = world.get_resource::<Renderer>().unwrap();
      unsafe {
        renderer.gl.bind_framebuffer(self.fb);
        renderer.resize([1920.0, 1080.0]);
        renderer.clear(self.fb);
      }
      world.add_resource(SceneAspect(self.size[0] / self.size[1]));
      scenerenderer_draw(world)?;
    }
    Ok(())
  }

  fn render(&mut self, _: &mut World, ui: &Ui) -> Result<()> {
    if self.open {
      ui.window("Scene")
        .opened(&mut self.open)
        .scroll_bar(false)
        .scrollable(false)
        .build(|| {
          Image::new(self.tex, self.size)
            .uv0([0.0, 1.0])
            .uv1([1.0, 0.0])
            .build(&ui);
          self.size = ui.window_size();
        });
    }
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
    if self.open {
      ui.window("Outline").opened(&mut self.open).build(|| {
        let [w, _] = ui.window_size();
        for (_, n) in world.query::<Name>() {
          ui.selectable(n.0);
        }
        ui.button_with_size("Add Entity", [w, 0.0]);
      });
    }
    Ok(())
  }

  fn title(&self) -> &'static str {
    "Outline"
  }

  fn open(&mut self) -> &mut bool {
    &mut self.open
  }
}
