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

- `rules/store-access.md` — applies when reading entities/components from the store
- `rules/conventions.md` — applies when writing game code (systems, frame time, assets)
- `rules/events.md` — applies when systems need to communicate with each other
- `rules/factories.md` — applies when writing factories (pure functions, no side effects)
- `rules/ownership.md` — applies when touching `pico_entity_store` or `src/util/estore.rs`
- `rules/git.md` — applies before committing changes
