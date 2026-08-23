# heroes-of-ironhold

A minimal Rust game project using [macroquad](https://docs.rs/macroquad/latest/macroquad/).

## Project Structure

- `src/` - Core library crate (`heroes-of-ironhold-core`): shared game logic, no platform knowledge
- `assets/` - Game assets: `images/` (PNG/TGA), `fonts/` (TTF), `audio/` (WAV/OGG)
- `clients/desktop/` - Desktop binary crate (`heroes-of-ironhold-desktop`)
- `clients/web/` - Web binary crate (`heroes-of-ironhold-web`): wasm32 target
- `crates/` - Support crates: `di-container`, `event-bus`, `pico-entity-store`, `tiled`
- `Cargo.toml` - Workspace root AND the `heroes-of-ironhold-core` package

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
