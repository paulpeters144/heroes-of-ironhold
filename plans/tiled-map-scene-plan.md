# Description
Create a new `battle_test` scene that loads `assets/tiled/test-scene/test.tmx` (via the `tiled` crate), renders the map tiles, spawns a hero knight, and lets the user control the knight on the map. The scene follows the conventions established by `src/scene/asset_preview/`: a scene module with a `Scene` impl plus ECS systems, registered in `SceneFactory`. Mostly new files; only minimal wiring changes to existing registration/ids files.

# TODO
- [ ] Add asset `Id` variants + manifests for the tiled map's TMX, TSX, and tileset image in `src/access/ids.rs`
- [ ] Create `src/scene/battle_test/mod.rs`
- [ ] Create `src/scene/battle_test/scene.rs`
- [ ] Create `src/scene/battle_test/sys_map_draw.rs`
- [ ] Create `src/scene/battle_test/sys_camera.rs`
- [ ] Create `src/scene/battle_test/sys_knight_controls.rs` (reuse `asset_preview`'s knight control systems where possible)
- [ ] Wire the new scene into `src/scene/mod.rs`
- [ ] Register `SceneId::BattleTest` in `src/scene/factory.rs`
- [ ] Set the startup scene (or add scene navigation) so the scene is reachable
- [ ] Verify build with `cargo check`

# TODO Explanation

## Add asset `Id` variants + manifests in `src/access/ids.rs`
The tiled crate needs the TMX/TSX as raw strings (`AssetKind::File`) and the tileset images as `Image` (`AssetKind::Image`), not `Texture`. `test.tmx` declares two tilesets, so all of the following are needed:
- `File`-kind variants (in the currently-empty `file` module + a `file` manifest):
  - `tiled/test-scene/test.tmx`
  - `tiled/test-scene/hoi-bg-pixelated.tsx`
  - `tiled/test-scene/hoi-chars.tsx`
- `Image`-kind variants for the tileset images (`tiled/test-scene/hoi-bg-pixelated.png` and `tiled/test-scene/hoi-chars.png`). The tiled crate keys `images` by filename (`"hoi-bg-pixelated.png"`, `"hoi-chars.png"`) and expects `Image`/`load_image`, so these cannot reuse existing `AssetKind::Texture` ids.
- Follow `AGENTS.md`: asset paths are relative to `assets/` (no `assets/` prefix), add a variant in the matching category module, register it in that module's manifest, access via typed accessors.

## Create `src/scene/battle_test/mod.rs`
Re-export the scene and systems, mirroring `src/scene/asset_preview/mod.rs`.

## Create `src/scene/battle_test/scene.rs`
Implement `BattleTestScene` following `AssetPreviewScene`:
- Fields: `cfg: &'static Config`, `assets: Assets`, `store: &'static EStore`, a `SystemAgg`, a `TiledMap` (built in `load`), plus knight/hero systems.
- `new(cfg, assets, store)`: build `SystemAgg`, register shared update/draw systems (`AnimationUpdateSystem`, `OffsetUpdateSystem`, `AttackEffectSystem`, `DrawSystem`) and new `MapDrawSystem` + `CameraSystem`. Systems needing textures/assets are registered in `load()` (per the `asset_preview` pattern of deferring `AttackEffectDrawSystem`).
- `load()`: `assets.preload(&[...])` the TMX/TSX files + tileset images; build `TiledMapCfg { tile_map, tile_sets: {"hoi-bg-pixelated.tsx": tsx, "hoi-chars.tsx": tsx}, images: {"hoi-bg-pixelated.png": image, "hoi-chars.png": image}, section_size: (16,16) }`; call `TiledMap::from_config(cfg)` and store the result. Spawn the hero via `HeroFactory::create_knight(KnightCfg { outfit: Knight1, sword: Sword1, shield: Shield1 })` + `store.add` (same hierarchy as `spawn_knight`) — variant 1, matching `asset_preview`'s default. Position the knight on the map (e.g. near map origin or a spawn point). The map is 125x32 tiles of 16px (2000x512 world px) with three layers (`far-mountains`, `mountains`, `ground`).
- `update(ctx)`: let `CameraSystem` set `ctx.cam_zoom`/`ctx.cam_pan` to follow the knight (instead of pinning to `1.0`/`ZERO` like `asset_preview`), then `self.agg.update(ctx)`.
- `draw(ctx)`: draw the map (via `MapDrawSystem`), then `self.agg.draw(ctx)` for entities.
- `draw_ui(ctx)`: empty (no UI for this scene).

## Create `src/scene/battle_test/sys_map_draw.rs`
A `Draw` system that renders the `TiledMap`. Compute the visible `Rect` from the camera (screen bounds adjusted by `ctx.cam_zoom`/`ctx.cam_pan`), call `map.get_sections(view)`, and for each section draw each `Some(TiledTile)` via `draw_texture_ex` with `source = tile.source`, at the section's world-space bounds origin. Map tiles are NOT ECS entities — they need this dedicated draw path (the shared `DrawSystem` only draws `Animation`/`StaticImage` components).

## Create `src/scene/battle_test/sys_camera.rs`
A `Update` system that reads the hero position (e.g. `store.first::<Knight>()` → body child position) and sets `ctx.cam_zoom`/`ctx.cam_pan` to center the camera on the hero. Replaces `asset_preview`'s static `cam_zoom=1.0`/`cam_pan=ZERO`. Confirm how `Manager::blit_target` consumes `ctx.cam_zoom`/`ctx.cam_pan` so the pan math matches the existing viewport logic.

## Create `src/scene/battle_test/sys_knight_controls.rs` (reuse where possible)
Reuse the existing `KnightControlSystem` (movement + attack state machine), `OffsetUpdateSystem`, `AnimationUpdateSystem`, `AttackEffectSystem`, and `DrawSystem` from `asset_preview`/`src/systems` — they are scene-agnostic and driven by abstract `Input` + ECS. Only create a new system file here if the tiled scene needs scene-specific control behavior (e.g. collision against `map.collide_statics`). Otherwise import/reuse the existing systems directly.

## Wire the new scene into `src/scene/mod.rs`
Add `pub mod battle_test;` and export the scene type alongside `asset_preview`.

## Register `SceneId::BattleTest` in `src/scene/factory.rs`
Add a `SceneId::BattleTest` variant and a `create` match arm that resolves `Assets` (transient) + `EStore` (singleton) from the DI container and calls `BattleTestScene::new(...)`, mirroring the existing `AssetPreview` arm.

## Set the startup scene / add navigation
Make `battle_test` the startup scene: change `Manager::new` to fire `ChangeSceneEvent(SceneId::BattleTest)` instead of `SceneId::AssetPreview`. `AssetPreview` stays registered but is no longer the entry point.

## Verify build with `cargo check`
Run `cargo check` (workspace) to confirm the new module and wiring compile. Note assets are loaded async/failure-tolerant, so missing/misnamed assets log a warning but do not crash.

# Open Questions

# Out of Scope
- We are not implementing generic Tiled object → entity conversion (the `tiled` crate leaves this unimplemented).
- We are not adding collision resolution against `map.collide_statics` (hero moves freely for now).
- We are not modifying the `pico_entity_store` crate or `src/util/estore.rs`.
- We are not adding new tilesets/art assets beyond what already exists under `assets/`.
- We are not changing the shared `DrawSystem`/`SystemAgg` core logic.
