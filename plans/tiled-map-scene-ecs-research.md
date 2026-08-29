# Research Topic
How the Tiled map system, the entity-component system (ECS), and the `asset_preview` scene work together — gathered as preparation for building a new scene that renders a Tiled map (`assets/tiled/`) and lets the user control a hero on it.

---

## 1. The Tiled crate (`crates/tiled`)

A hand-written parser that converts Tiled `.tmx`/`.tsx` XML into macroquad-ready Rust structs. It does **no I/O** — the caller supplies the raw TMX/TSX strings and the already-loaded tileset `Image`s.

### Entry point: `TiledMap::from_config`

```rust
pub struct TiledMapCfg<'a> {
    pub tile_map: String,                    // raw .tmx XML text
    pub tile_sets: HashMap<String, String>,  // raw .tsx XML, keyed by the `source` path in the TMX
    pub images: &'a HashMap<String, Image>,  // tileset images, keyed by image `source` filename from the TSX
    pub section_size: (u32, u32),            // width/height of each section in tiles (e.g. (16,16))
}
```

`TiledMap::from_config(cfg) -> Result<TiledMap, TiledError>` (`crates/tiled/src/tiled_map.rs`).

### Key result structs

- **`TiledMap`** (`tiled_map.rs`) — `tilesets: Vec<TiledTileset>`, `tile_size: (u16,u16)`, `map_size: (u32,u32)`, `section_size`, `layers: Vec<TiledLayer>`, `collide_statics: Vec<CollideStatic>`, `patrol_points: Vec<Vec2>`.
- **`TiledTileset`** (`tiled_tileset.rs`) — `firstgid`, `tile_width`, `tile_height`, `columns`, `image: Image`, `texture: Option<Texture2D>`. The constructor builds a `Texture2D::from_image` and sets `FilterMode::Nearest`.
- **`TiledLayer`** (`tiled_layer.rs`) — `name: String`, `sections: Vec<TiledSection>`.
- **`TiledSection`** (`tiled_section.rs`) — `grid_pos: (u32,u32)`, `bounds: Rect` (world space), `tiles: Vec<Option<TiledTile>>` (flat row-major; `None` = empty tile).
- **`TiledTile`** (`tiled_tile.rs`) — `texture: Texture2D`, `source: Rect` (pixel rect into the tileset texture).
- **`CollideStatic`** (`collide_static.rs`) — a newtype over `Rect`, extracted from the `"collide"` object group.

### Culling

`TiledMap::get_sections(view: Rect) -> Vec<&TiledSection>` flattens all layers' sections and filters by `rects_overlap(section.bounds, view)`. The map is split into `section_size`-sized chunks precisely so you only draw what the camera sees.

### Tile resolution

GIDs from the CSV layer data are resolved against the tileset with the largest `firstgid <= gid` (`resolve_tile`):
`local = gid - firstgid`, `src_x = (local % columns) * tw`, `src_y = (local / columns) * th`.

### Parser limitations (important)

- Only **orthogonal** maps with **CSV-encoded** tile data are supported (`tiled_map.rs` `from_config`).
- Tile layers are parsed into drawable `TiledTile`s.
- Object layers are parsed into raw shapes only where they have hard-coded names: `objectgroup name="collide"` → `CollideStatic(Rect)`, `objectgroup name="patrol"` → `Vec2` points. Other object groups are ignored. Converting generic objects to game-ready entities is **not yet implemented** (`README.md`).
- The tileset image key is derived via `Path::new(&src).file_name()` (`parse_tsx` in `tiled_tileset.rs`), so the `images` map must be keyed by the *filename* (e.g. `"tile-set.png"`), not the full relative path.
- The `tile_sets` map is keyed by the exact `source` path referenced in the TMX (e.g. `"tile-set.tsx"`).

### Tests

`crates/tiled/tests/tiled_map.rs` demonstrates the full flow with sample data and `#[macroquad::test]` async tests; `section_size (16,16)`.

---

## 2. Assets under `assets/tiled/`

- `scene-1.tmx` — 40x18 map, `tilewidth=16`, references `tile-set.tsx` (`firstgid=1`). Has a `"collide"` objectgroup (3 rects) and a `"patrol"` objectgroup (2 points).
- `tile-set.tsx` — 16x16 tiles, 1320 tiles, 33 columns, image `../images/tile-set.png` (528x640).
- `test-scene/` — a second, more elaborate example: `test.tmx` + `hoi-bg-pixelated.tsx`/`.png` (16x16) and `hoi-chars.tsx`/`.png` (64x64 tiles; a character tileset).
- `heroes-of-ironhold.tiled-project` / `.tiled-session`, `test.tiled-project` / `.tiled-session` — editor state only (not needed at runtime).

### Implications for loading a map in-game

- The TMX and TSX must be loaded as **strings** (`load_string` → `AssetKind::File`) and the tileset image as an **`Image`** (`load_image` → `AssetKind::Image`).
- For `scene-1.tmx`: `tile_map` = `assets/tiled/scene-1.tmx` text; `tile_sets` = `{"tile-set.tsx": <tsx text>}`; `images` = `{"tile-set.png": <image>}` (filename only).
- The tileset image source `../images/tile-set.png` resolves to filename `tile-set.png`, matching the existing `images/tile-set.png` asset path in `src/access/ids.rs`.

---

## 3. The ECS (`pico-entity-store` + `EStore` wrapper)

`crates/pico-entity-store` is a tiny, no-macro, type-erased entity-component store. `src/util/estore.rs` wraps it in `EStore` (which `Deref`s to the inner `EntityStore`) so scenes hold `&'static EStore`.

### Core model

- Entities are identified by a `usize` id; components are stored per-type in contiguous `Vec<T>` buffers.
- A **parent/child hierarchy** lets an entity attach other entities as children (used to compose the knight from body + shield + sword).

### API used in the codebase

- `store.add(target, &children) -> EntityRef` — `target: impl IntoAdd<T>`. Pass a **component by value** to create a new entity, or a `Ref`/`RefMut` to attach children to an existing entity. `children: &[ChildSource]` is usually built with the `children!` macro or the `IntoChild::into_child()` method. Attachment is all-or-nothing.
- `store.first::<T>()` / `first_mut::<T>()` — first live entity of a component type.
- `store.get_by_id::<T>(id)` / `get_by_id_mut::<T>(id)` — look up a specific entity by id.
- `store.all::<T>()` / `all_mut::<T>()` — iterate every component of a type (used by systems).
- `store.update::<T>(entity_ref, |t| {...})` — mutate a component in place.
- `store.remove(&[EntityRef])` — removes entities plus all descendants (recursive).
- `store.get_child::<T>(parent)` / `get_child_mut::<T>(parent)` — first direct child of a given component type.
- `store.children(parent)`, `store.parent(child)`, `store.descendants(entity)`.

`EntityRef { id, type_id }` is the lightweight, `Copy` handle (`entity_ref.rs`); `Ref`/`RefMut` are guards holding the store lock.

### Components (defined in `src/entity/`)

- `Animation` (`animation.rs`) — texture, position, frame dims, frame count, current frame, tint, flip_x, dest_size, visible, z_idx.
- `StaticImage` (`static_image.rs`) — single-image drawable.
- `Knight`, `Shield`, `Sword` (`knight/components.rs`) — marker/component structs used as the knight entity and its parts.
- `Effect` (`knight/effect.rs`) — transient attack effect (age/lifetime/origin).

### Hierarchy pattern (from `factory_hero.rs` + `asset_preview/scene.rs::spawn_knight`)

`HeroFactory::create_knight(KnightCfg)` returns `KnightParts { body: Animation, shield: Shield, shield_image: StaticImage, sword: Sword, sword_animation: Animation }`. The scene then:
1. `store.add(parts.shield, &[parts.shield_image.into_child()])` — Shield entity owns a StaticImage child.
2. `store.add(parts.sword, &[parts.sword_animation.into_child()])` — Sword entity owns an Animation child.
3. `store.add(Knight, &[parts.body.into_child(), shield_ref.into_child(), sword_ref.into_child()])` — the Knight is the root, children are the body Animation plus the Shield and Sword entities.

Systems then walk this hierarchy: e.g. `OffsetUpdateSystem` reads `first::<Knight>()` → `get_child::<Animation>` (body position) and positions the shield/sword children relative to it.

---

## 4. The `asset_preview` scene as an example (`src/scene/asset_preview/`)

This is the template to copy for a new scene.

### `Scene` trait (`src/scene/traits.rs`)

```rust
pub trait Scene: 'static {
    fn load(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>>;
    fn update(&mut self, ctx: &mut Context);
    fn draw(&self, ctx: &Context);
    fn draw_ui(&self, _ctx: &Context) {}
    fn dispose(&mut self) {}
}
```

### `AssetPreviewScene` structure (`scene.rs`)

- Holds `cfg: &'static Config`, `assets: Assets`, `store: &'static EStore`, plus textures, selection state, a `KnightControlSystem`, and a `SystemAgg`.
- `new(cfg, assets, store)` builds a `SystemAgg` and registers **update** systems (`AnimationUpdateSystem`, `OffsetUpdateSystem`, `AttackEffectSystem`) and the **draw** system (`DrawSystem`). Systems that need textures (like `AttackEffectDrawSystem`) are added later in `load()`.
- `load()` — `assets.preload(&[...])` all needed textures; constructs `AttackEffectDrawSystem` and adds it; `spawn_knight(...)`; positions the knight.
- `update(ctx)` — sets `ctx.cam_zoom = 1.0`, `ctx.cam_pan = Vec2::ZERO` (no camera movement here); processes input (focus mode switching / box cycling); calls `self.agg.update(ctx)`.
- `draw(ctx)` — fills the background, calls `self.agg.draw(ctx)`.
- `draw_ui(ctx)` — creates `UI`, `ui.begin(ctx)`, draws the selection boxes, `ui.end()`.

### Systems (`sys_*.rs`)

Each system holds `store: &'static EStore` and implements `Update` (or `Draw`) from `src/systems/system_agg.rs`. Examples:

- `AnimationUpdateSystem` — frame-advances every `Animation` with `running == true` on a `FRAME_DURATION` timer.
- `OffsetUpdateSystem` — reads the knight body position + current frame, then repositions the shield/sword children using `Knight::offsets()` (`FrameOffsets`).
- `KnightControlSystem` — reads `input::down/down_once`, moves the body (`MOVE_SPEED * ctx.dt`), walks through `WALK_FRAMES`, and drives a `Thrust`/`Swipe` attack state machine (`AttackPhase`).
- `AttackEffectSystem` / `AttackEffectDrawSystem` — spawn transient `Effect` components on strike frames and draw glow textures.
- `DrawSystem` (`src/systems/sys_draw.rs`) — collects all `Animation` + `StaticImage` components, sorts by `z_idx`, draws them.

`SystemAgg` is a simple container of `Vec<Box<dyn Update>>` and `Vec<Box<dyn Draw>>` with `add_update`, `add_draw`, `remove_*` (by `TypeId`), `clear`, `update(ctx)`, `draw(ctx)`.

### Scene lifecycle & registration

- `SceneId` enum and `SceneFactory` live in `src/scene/factory.rs`. `SceneFactory::create(id)` resolves `Assets` (transient) and `EStore` (singleton) from the DI container and calls `AssetPreviewScene::new(...)`. **Only `AssetPreview` exists today** — a new scene means adding a `SceneId` variant + a `create` match arm.
- `Manager` (`src/manager/mod.rs`) owns `SceneState` (Idle / Transition). Scene changes go through the `ChangeSceneEvent` event on the `EventBus`. `Manager::new` currently fires `ChangeSceneEvent(SceneId::AssetPreview)` at startup.
- `init()` (`src/lib.rs`) builds the DI container singletons (`Config`, `EventBus`, `EStore`, `SystemAgg`, `GameRenderTarget`, `GameCamera`) + transient `Assets`, then constructs the `Manager`.

### Rendering pipeline (important for a map scene)

`Manager::draw`:
1. `begin_scene_pass()` — `set_camera(&self.camera.camera)` (a `Camera2D` with an overscanned render target), clear background.
2. `draw_scene(ctx)` — the scene's `draw` runs in world/camera space.
3. `begin_screen_pass()` — `set_default_camera()`.
4. `blit_target(ctx)` — blits the render-target texture to the screen, using `ctx.cam_zoom` and `ctx.cam_pan` to pick the source rect.
5. `draw_ui(ctx)` — sets a screen-space UI camera and calls `scene.draw_ui`.

`Context { dt, cam_zoom, cam_pan }` is the per-frame data; `dt` is set by the `Manager` from `get_frame_time()`. A map scene with a camera would drive `ctx.cam_zoom`/`ctx.cam_pan` in `update` (the current scene just pins them to `1.0` / `Vec2::ZERO`).

---

## 5. Input & Assets plumbing

### Input

- The clients (`clients/desktop/src/main.rs`, `clients/web/src/main.rs`) map macroquad keys to the abstract `Input` enum via `input::set(...)`:
  `Enter`→`KeyCode::Enter`, `Up/Down/Left/Right`→arrow keys, `Jump`→`Space`, `Attack`→`X`.
- The core reads abstract input via `input::down`, `down_once`, `up_once`, `down_once_every` (`src/input/mod.rs`). `input::end_frame(dt)` is called by `update` each frame.
- `Input` variants: `Enter, Up, Down, Left, Right, Jump, Attack` (non-exhaustive).

### Assets (`src/access/assets.rs` + `src/access/ids.rs`)

- `Assets::preload(&[ids])` loads based on `AssetId::kind()`; accessors `texture`, `image`, `file`, `font`, `sound`, `shader` panic if not preloaded.
- `AssetId` trait: `path() -> String`, `kind() -> AssetKind`; `AssetKind` = `Texture | Image | Font | Sound | File | Shader`.
- Current `ids` modules: `font` (2 variants), `shader` (4 variants), `images` (Knight/Enemy/Megabot/Pointer/TileSet — all `AssetKind::Texture`), `texture`/`sound`/`file` are **empty enums**.

### Gap to fill for a Tiled scene

To feed `TiledMapCfg` the scene needs:
1. **`File`-kind ids** for `tiled/scene-1.tmx` and `tiled/tile-set.tsx` (the `file` module in `ids.rs` is currently empty — add variants + manifest).
2. An **`Image`-kind id** for the tileset image (currently all `images` variants are `AssetKind::Texture`; the tiled crate needs `Image`/`load_image`, and its `images` map is keyed by filename `"tile-set.png"`).

Per the `AGENTS.md` asset convention, assets are referenced by path relative to `assets/` (e.g. `tiled/scene-1.tmx`, `images/tile-set.png`), and adding an asset means adding an `Id` variant in the matching category module in `src/access/ids.rs`.

---

## 6. Notes for building the new "Tiled map + controllable hero" scene

1. **New scene module** `src/scene/tiled_map/` mirroring `asset_preview`: `scene.rs` + `sys_*.rs`, re-exported from `src/scene/mod.rs`.
2. **Register in `SceneFactory`**: add `SceneId::TiledMap` (or similar) + a `create` arm resolving `Assets` + `EStore` the same way.
3. **Bootstrap navigation**: `Manager::new` fires `ChangeSceneEvent(SceneId::AssetPreview)`. Either switch the startup scene or add input/event-driven switching (a `ChangeSceneEvent` can be fired from anywhere via the `EventBus`).
4. **Load the map in `load()`**:
   - `preload` the TMX/TSX as `File` and the tileset as `Image`.
   - Build `TiledMapCfg { tile_map, tile_sets: {"tile-set.tsx": tsx}, images: {"tile-set.png": image}, section_size: (16,16) }`.
   - `TiledMap::from_config(cfg)`; store the result on the scene (or in a resource).
5. **Draw the map** in `draw` (or a `MapDrawSystem`): compute the visible `Rect` from the camera, call `map.get_sections(view)`, and for each section draw each `Some(TiledTile)` via `draw_texture_ex` with `source = tile.source` at the section's world position. The `DrawSystem` only draws `Animation`/`StaticImage` ECS components, so map tiles need their own draw path (not necessarily entities).
6. **Hero control**: reuse the existing `HeroFactory` + `KnightControlSystem`/`OffsetUpdateSystem`/`AnimationUpdateSystem`/`DrawSystem` to spawn and drive a knight. These systems are ECS-driven and scene-agnostic; `KnightControlSystem` currently uses abstract `Input::{Left,Right,Up,Down,Attack}` with a fixed movement speed, so it works as-is in a new scene.
7. **Collision**: `map.collide_statics` (`Vec<CollideStatic>` of `Rect`s) is available for the hero to collide against (no collision resolution exists yet — would be new work).
8. **Camera**: to follow the hero, set `ctx.cam_pan` (and optionally `ctx.cam_zoom`) in `update`; the `Manager::blit_target` already consumes these to move the viewport.
