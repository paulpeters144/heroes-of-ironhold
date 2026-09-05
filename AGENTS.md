# AGENTS.md

A minimal Rust game project using macroquad.

## Project Structure

- `src/` - Core library crate (`heroes-of-ironhold-core`): shared game logic, no platform knowledge
- `assets/` - Game assets: `images/` (PNG/TGA), `fonts/` (TTF), `audio/` (WAV/OGG)
- `clients/desktop/` - Desktop binary crate (`heroes-of-ironhold-desktop`)
- `clients/web/` - Web binary crate (`heroes-of-ironhold-web`): wasm32 target
- `Cargo.toml` - Workspace root AND the `heroes-of-ironhold-core` package
- `docs/` - Reference docs (see [Docs](#docs) below)

## Assets

- Assets live in `assets/` and are referenced in code by path relative to that folder, e.g. `images/player.png` (no `assets/` prefix).
- `init()` calls `set_pc_assets_folder("assets")` before loading, so the same paths work on desktop and web.
- To add an asset: drop the file in the matching `assets/` subdir, add an `Id` variant in the matching category module in `src/access/ids.rs` (e.g. `sound::Id`), and register it in that module's manifest (e.g. `sound::SOUNDS`). Access it via the typed accessors, e.g. `game.assets.sound(sound::Id::Jump)`. Image keys must stay the filename (the `tiled` crate looks up `assets.images` by filename).
- Loading is async and failure-tolerant: a missing asset logs a warning and the game keeps running.
- Desktop: paths resolve against the current working directory, so run from the repo root.
- Web: `clients/web/build.py` copies `assets/` into `dist/assets/` so the wasm can fetch them over HTTP.

## Build & Run

Desktop:
```bash
cargo run -p heroes-of-ironhold-desktop
```

Web (requires `wasm32-unknown-unknown` target):
```bash
rustup target add wasm32-unknown-unknown  # one-time setup
python3 clients/web/build.py
python3 clients/web/serve.py              # serve on port 3001
```

## Docs

- [macroquad Documentation](docs/macroquad-docs.md) - links to macroquad's own docs, GitHub, and examples
- [Drawing Graphics in macroquad](docs/macroquad-graphics.md) - shapes, outlines, and alpha
- [Writing Shaders in macroquad](docs/macroquad-shaders.md) - shaders and materials
- [macroquad Camera2D Zoom](docs/camera-scaling.md)
- [Camera](docs/camera.md) - how the game camera pipeline and `cam_zoom`/`cam_target` work
- [tiled crate](crates/tiled/README.md) - TMX map parser and renderer

## Rules

- Never commit changes unless the user explicitly asks you to.
- All systems start with sys_*.rs for example: sys_animation.rs
- Never call `get_frame_time()`; read the frame delta time from `Context` (`ctx.dt`). `Manager` updates it once per frame.
- Never call `Assets` accessors (`get_font`, `sound`, `texture`, etc.) in `update()` or `draw()` methods. Extract all needed assets during construction (e.g. `new()` or `factory()`) and store the results on the struct.
- Never modify the `pico_entity_store` crate unless explicitly told to.
- Never modify `src/util/estore.rs` unless explicitly told to.
- Read entities and components from the store with the fluent accessor chain (`first`, `get_child`, ...). Do NOT write utility methods on the store, add new store methods, or mutate the store internals just to look something up. The store already exposes everything you need; traverse the hierarchy by chaining guards. Example:
  ```rust
  let current_frame = self
      .store
      .first::<PlayerOne>()
      .and_then(|player| self.store.get_child::<Knight>(&player))
      .and_then(|knight| self.store.get_child::<Animation>(&knight))
      .map(|a| a.current_frame);
  ```
  Key accessor methods (see `crates/pico-entity-store/src/store.rs`): `first::<T>()`/`first_mut::<T>()` find the first live component of type `T`; `get_child::<T>(&parent)`/`get_child_mut::<T>(&parent)` find the first direct child of `parent` of type `T`; `all::<T>()`/`all_mut::<T>()` iterate every component of type `T`; `get_by_id::<T>(id)`/`get_by_id_mut::<T>(id)` fetch by numeric id; `parent`/`children`/`descendants` navigate the hierarchy. Guards deref to the component (`&T`/`&mut T`) and expose `.id()` and `.entity_ref()`. Never modify the store for reads; mutate in place via a mutable guard instead.
