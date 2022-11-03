# Project Setup

## Create a Cargo project

You can use Phosphor with any other Rust dependencies with Cargo.

Run the following commands to setup your project.

```sh
cargo new my_game
cd my_game
```

## Add Phosphor as a dependency

Phosphor is not yet on crates.io. For now you can include it locally by adding this to your `Cargo.toml`.

```toml
[dependencies]
phosphor = { path = "../phosphor" }
```

## Build Phosphor

Add the following code to your `src/main.rs`.

```rs
use phosphor::{Engine,Result};

fn main() -> Result<()> {
  Engine::new().run()
}
```

Now run your project with `cargo run`.

## Logging

The previous code will only show a black window for now, but first lets add a logger to help us debug in the future.

Add a logger of your choice to the `Cargo.toml` (This example uses [env_logger](https://crates.io/crates/env_logger)).

```toml
[dependencies]
phosphor = { path = "../phosphor" }
env_logger = "0.9"
```

```rs
use phosphor::{Engine, Result};
use phosphor::log::LevelFilter;

fn main() -> Result<()> {
  env_logger::builder().filter_level(LevelFilter::Info).init();
  Engine::new().run()
}
```

Now after running `cargo run` you should see something similar to the following in your terminal.

```
[2022-11-03T20:50:09Z INFO  winit::platform_impl::platform::x11::window] Guessed window scale factor: 1.1666666666666667
[2022-11-03T20:50:09Z INFO  phosphor::gfx] Created renderer: NVIDIA GeForce RTX 3060 Ti/PCIe/SSE2
```

Now we have our project setup we can begin creating our game.
