# UI

We can use the `phosphor_ui` crate to debug our game using [imgui](https://github.com/ocornut/imgui). First enable the `uirenderer` system;

```rs
use phosphor_ui::uirenderer;

fn main() -> Result<()> {
  ...
  Engine::new()
    .add_system(Stage::Start, &uirenderer)
    .run()
}
```

Now during the draw stage, we have access to the `Ui` resource, so lets try draw some text.

```rs
use phosphor_ui::{uirenderer, imgui::Ui};

fn main() -> Result<()> {
  ...
  Engine::new()
    .add_system(Stage::Start, &uirenderer)
    .add_system(Stage::Draw, &draw_ui)
    .run()
}

fn draw_ui(world: &mut World) -> Result<()> {
  let ui = world.get_resource::<Ui>().unwrap();
  ui.text("Hello ui!");
  Ok(())
}
```
