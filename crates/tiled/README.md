# tiled

Parses Tiled `.tmx` maps into game-ready Rust structs for macroquad games. The
crate splits each tile layer into a grid of *sections* so you can cheaply cull
rendering to only what a camera can see.

The crate does **no I/O** — you supply the TMX/TSX XML text and the already-loaded
tileset images, and it produces a `TiledMap`.

## Adding to your project

```toml
[dependencies]
tiled = { path = "crates/tiled" }
macroquad = { version = "0.4", features = ["audio"] }
```

## Usage

1. Load the TMX and TSX files as strings and the tileset images as
   `macroquad::texture::Image` (e.g. with `load_image`).
2. Build a `TiledMapCfg`.
3. Call `TiledMap::from_config(cfg)`.
4. Each frame, query the sections overlapping the camera view with
   `get_sections(rect)` and draw only those.

```rust
use std::collections::HashMap;
use macroquad::math::Rect;
use macroquad::prelude::*;
use tiled::{TiledMap, TiledMapCfg};

struct Assets {
    tmx: String,
    tsx: String,
    tileset_image: Image,
}

async fn build_map(assets: Assets) -> Result<TiledMap, tiled::TiledError> {
    let cfg = TiledMapCfg {
        tile_map: assets.tmx,
        tile_sets: HashMap::from([("bg-1.tsx".into(), assets.tsx)]),
        images: HashMap::from([("bg-1.png".into(), assets.tileset_image)]),
        // Each section holds a 16x16 chunk of tiles.
        section_size: (16, 16),
    };
    TiledMap::from_config(cfg)
}

fn draw_visible(map: &TiledMap, camera: &Camera2D) {
    let view = Rect::new(
        camera.target.x - screen_width() / 2.0,
        camera.target.y - screen_height() / 2.0,
        screen_width(),
        screen_height(),
    );
    for section in map.get_sections(view) {
        // Draw section.tiles to the screen using section.bounds as the
        // world-space rectangle and map.images["bg-1.png"] as the source.
    }
}
```

## Configuration

`TiledMapCfg` fields:

| Field | Type | Meaning |
|-------|------|---------|
| `tile_map` | `String` | Raw `.tmx` XML |
| `tile_sets` | `HashMap<String, String>` | Raw `.tsx` XML, keyed by the `source` path referenced in the TMX (e.g. `"bg-1.tsx"`) |
| `images` | `HashMap<String, Image>` | Tileset images, keyed by the image `source` path from the TSX (e.g. `"bg-1.png"`) |
| `section_size` | `(u32, u32)` | Width/height of each section in tiles |

## Models

- `TiledMap` — top-level map: `images`, `tile_size`, `map_size`, `section_size`, `layers`.
- `Layer` — a tile layer: `name`, `sections`.
- `Section` — a chunk of a layer: `grid_pos` (col, row), `bounds` (world-space `Rect`), `tiles` (flat, row-major local tile IDs; `0` = empty).
- `TiledRawObject` / `TiledRawObjectGroup` / `TiledRawShape` — parsed object layers (conversion to game-ready objects is not yet implemented).

## Notes

- Tile IDs stored in `Section.tiles` are the raw CSV values from Tiled (global IDs, `0` = empty).
- Only orthogonal maps with CSV-encoded tile data are currently parsed.
- Object layers are parsed into `TiledRawObject` structures but the conversion to `TiledObject` is a future step.

## Tests

```bash
cargo test -p tiled
```
