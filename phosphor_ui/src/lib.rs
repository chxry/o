use std::{fs, mem};
use std::time::Instant;
use phosphor::{Result, Event, grr};
use phosphor::gfx::{Renderer, Shader, Texture};
use phosphor::ecs::{World, Stage};
use phosphor::math::Mat4;
use phosphor::log::warn;

pub use imgui;

pub type Textures = imgui::Textures<Texture>;

struct UiRenderer {
  imgui: imgui::Context,
  platform: imgui_winit_support::WinitPlatform,
  shader: Shader,
  vert_arr: grr::VertexArray,
  last_frame: Instant,
}

pub fn uirenderer(world: &mut World) -> Result<()> {
  let renderer = world.get_resource::<Renderer>().unwrap();
  let mut imgui = imgui::Context::create();
  imgui.set_ini_filename(None);

  let mut fonts = imgui.fonts();
  fonts.add_font(&[imgui::FontSource::TtfData {
    data: &fs::read("res/roboto.ttf")?,
    size_pixels: 16.0,
    config: None,
  }]);
  let font_tex = fonts.build_rgba32_texture();
  let mut textures = imgui::Textures::new();
  fonts.tex_id = textures.insert(Texture::new(
    renderer,
    font_tex.data,
    font_tex.width,
    font_tex.height,
  )?);
  drop(fonts);

  let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
  platform.attach_window(
    imgui.io_mut(),
    renderer.context.window(),
    imgui_winit_support::HiDpiMode::Locked(1.0),
  );
  let shader = Shader::new(renderer, "res/imgui.vert", "res/imgui.frag")?;
  let vert_arr = unsafe {
    renderer.gl.create_vertex_array(&[
      grr::VertexAttributeDesc {
        location: 0,
        binding: 0,
        format: grr::VertexFormat::Xy32Float,
        offset: 0,
      },
      grr::VertexAttributeDesc {
        location: 1,
        binding: 0,
        format: grr::VertexFormat::Xy32Float,
        offset: 8,
      },
      grr::VertexAttributeDesc {
        location: 2,
        binding: 0,
        format: grr::VertexFormat::Xyzw8Unorm,
        offset: 16,
      },
    ])?
  };
  world.add_resource(UiRenderer {
    imgui,
    platform,
    shader,
    vert_arr,
    last_frame: Instant::now(),
  });
  world.add_resource(textures);
  world.add_event_handler(&uirenderer_event);
  world.add_system(Stage::PreDraw, &uirenderer_predraw);
  world.add_system(Stage::PostDraw, &uirenderer_draw);
  Ok(())
}

fn uirenderer_event(world: &mut World, event: &Event<()>) -> Result<()> {
  let renderer = world.get_resource::<Renderer>().unwrap();
  let r = world.get_resource::<UiRenderer>().unwrap();
  r.platform
    .handle_event(r.imgui.io_mut(), renderer.context.window(), event);
  Ok(())
}

fn uirenderer_predraw(world: &mut World) -> Result<()> {
  let renderer = world.get_resource::<Renderer>().unwrap();
  let r = world.get_resource::<UiRenderer>().unwrap();
  r.platform
    .prepare_frame(r.imgui.io_mut(), renderer.context.window())?;
  world.add_resource::<imgui::Ui>(unsafe { mem::transmute(r.imgui.frame()) });
  Ok(())
}

fn uirenderer_draw(world: &mut World) -> Result<()> {
  if let Some(ui) = world.take_resource::<imgui::Ui>() {
    let renderer = world.get_resource::<Renderer>().unwrap();
    let textures = world.get_resource::<Textures>().unwrap();
    let r = world.get_resource::<UiRenderer>().unwrap();
    let io = r.imgui.io_mut();
    let [width, height] = io.display_size;
    let now = Instant::now();
    io.update_delta_time(now - r.last_frame);
    r.last_frame = now;

    r.platform.prepare_render(&ui, renderer.context.window());
    let draw_data = ui.render();
    for draw_list in draw_data.draw_lists() {
      unsafe {
        let vert_buf = renderer.gl.create_buffer_from_host(
          grr::as_u8_slice(draw_list.vtx_buffer()),
          grr::MemoryFlags::empty(),
        )?;
        let idx_buf = renderer.gl.create_buffer_from_host(
          grr::as_u8_slice(draw_list.idx_buffer()),
          grr::MemoryFlags::empty(),
        )?;
        r.shader.bind(renderer);
        r.shader.set_mat4(
          renderer,
          0,
          &Mat4::orthographic_rh(0.0, width, height, 0.0, 0.0, 1.0),
        );
        renderer.gl.bind_vertex_array(r.vert_arr);
        renderer.gl.bind_index_buffer(r.vert_arr, idx_buf);
        renderer.gl.bind_vertex_buffers(
          r.vert_arr,
          0,
          &[grr::VertexBufferView {
            buffer: vert_buf,
            offset: 0,
            stride: std::mem::size_of::<imgui::DrawVert>() as _,
            input_rate: grr::InputRate::Vertex,
          }],
        );
        let mut i = 0;
        for cmd in draw_list.commands() {
          if let imgui::DrawCmd::Elements { count, cmd_params } = cmd {
            match textures.get(cmd_params.texture_id) {
              Some(tex) => tex.bind(renderer),
              None => warn!("Texture {} does not exist.", cmd_params.texture_id.id()),
            };

            renderer.gl.set_scissor(
              0,
              &[grr::Region {
                x: cmd_params.clip_rect[0] as _,
                y: (height - cmd_params.clip_rect[3]) as _,
                w: (cmd_params.clip_rect[2] - cmd_params.clip_rect[0])
                  .abs()
                  .ceil() as _,
                h: (cmd_params.clip_rect[3] - cmd_params.clip_rect[1])
                  .abs()
                  .ceil() as _,
              }],
            );
            renderer.gl.draw_indexed(
              grr::Primitive::Triangles,
              grr::IndexTy::U16,
              i..i + count as u32,
              0..1,
              0,
            );
            i += count as u32;
          }
        }
        renderer.gl.delete_buffer(vert_buf);
        renderer.gl.delete_buffer(idx_buf);
      }
    }
  }
  Ok(())
}
