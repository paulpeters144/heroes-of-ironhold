# Camera

This document explains how the camera works in Heroes of Ironhold. It covers the
rendering pipeline, the `Context.cam_target` value, and how a scene drives the
camera (with `battle_test` as the working example).

## Overview

The game renders the scene into an offscreen render target, then blits a
sub-rectangle of that buffer to the screen.

1. **`GameCamera`** (`src/util/camera.rs`) — owns the render target and the
   `display_rect` (the logical view centered inside the larger overscanned
   buffer). It exposes `camera_at(target)` which returns a macroquad `Camera2D`
   centered on a world-space point.
2. **`Context.cam_target`** (`src/lib.rs`) — a per-frame world-space point that
   the scene sets each frame; the `Manager` uses it to re-center the `Camera2D`
   before rendering. `Context.cam_zoom` still exists (default `1.0`) but is not
   currently driven by any scene.

`Manager` (`src/manager/mod.rs`) owns the flow between the two.

## The render pipeline

`Manager::draw` (`src/manager/mod.rs:97-106`) runs four passes in order:

1. `begin_scene_pass(ctx)` (`:106`) — builds a `Camera2D` centered on
   `ctx.cam_target` via `camera.camera_at(...)`, `set_camera(...)`, then
   `clear_background(BLACK)`.
2. `draw_scene(ctx)` (`:111`) — calls `scene.draw(ctx)`.
3. `begin_screen_pass()` (`:120`) — `set_default_camera()`, clear.
4. `blit_target()` (`:159`) — draws the render target to the screen using a
   fixed central source rect, then `draw_ui(ctx)`.

### `blit_target`: a fixed central view

```rust
let src = Rect::new(
    (self.cfg.rt_width() - self.cfg.v_width) * 0.5,
    (self.cfg.rt_height() - self.cfg.v_height) * 0.5,
    self.cfg.v_width,
    self.cfg.v_height,
);
draw_texture_ex(&self.camera.render_target.texture, offset.x, offset.y, WHITE,
    DrawTextureParams { dest_size: ..., source: Some(src), flip_y: true, ..Default::default() });
```

The source rect is always the central `576 x 324` sub-rectangle of the `1152 x 648`
buffer, so it can never run off the render target. The visible world is chosen by
moving the scene camera, not by shifting this rect.

### The UI pass has its own camera

`draw_ui` (`src/manager/mod.rs:125-157`) builds a `Camera2D` on the fly with
`zoom = vec2(2.0 / v_width, 2.0 / v_height)` and a viewport sized for letterboxing
(`scale = min(screen_w / v_w, screen_h / v_h)`), then draws `scene.draw_ui(ctx)`.
UI rendering is unaffected by `cam_target`.

## `GameCamera` setup

`src/util/camera.rs`:

- `GameRenderTarget` (`:7-24`) creates the render target at `rt_width x rt_height`
  with `FilterMode::Nearest`.
- `GameCamera` (`:26-55`) stores:
  - `display_rect = Rect::new(-(rt_w - v_width)*0.5, -(rt_h - v_height)*0.5, rt_w, rt_h)`
    — the logical view centered inside the larger buffer.
  - `render_target` (cloned from `GameRenderTarget`).
  - `camera_at(target)` (`:35-41`) builds a fresh `Camera2D` centered on `target`:
    `Camera2D::from_display_rect(display_rect)` (1:1 world-to-pixel), then sets
    `camera.target = target` and `camera.render_target = Some(render_target)`.

Because `from_display_rect` maps 1 world unit to 1 render-target pixel, moving the
camera `target` shifts the capture window one render-target pixel per world unit.
A world point `p` maps to render-target pixel
`(p.x - target.x + rt_w/2, p.y - target.y + rt_h/2)`, so the buffer always captures
the world rectangle `[target.x - 576, target.x + 576] x [target.y - 324, target.y + 324]`.

## Config

`src/util/config.rs`:

- `v_width = 640.0 * 0.9 = 576.0`, `v_height = 360.0 * 0.9 = 324.0`.
- `rt_overscan = 2.0`; `rt_width() = 1152.0`, `rt_height() = 648.0`.
- Window `1280x720`.

## `Context`

`src/lib.rs:32-36`:

```rust
pub struct Context { pub dt: f32, pub cam_zoom: f32, pub cam_target: Vec2 }
```

Defaults set in `init()`: `cam_zoom: 1.0`, `cam_target: Vec2::ZERO` (`:63-67`).
`dt` is set by `Manager::update` from `get_frame_time()`
(`src/manager/mod.rs:72`); systems read `ctx.dt` rather than calling
`get_frame_time()` themselves.

## Driving the camera from a scene

### `battle_test` (follow the hero)

The follow camera is an `Update` system registered in `BattleTestScene::load`
(`src/scene/battle_test/scene.rs:151-157`):

```rust
agg.add_update(CameraSystem::new(store, cfg.v_width, cfg.v_height, map_w, map_h));
```

`CameraSystem` (`src/scene/battle_test/sys_camera.rs`):

- `locate_knight()` (`:25-30`) finds the first `Knight` entity and reads its child
  `Animation.position` (the body sprite position, in world space).
- `update(ctx)` (`:41-51`) centers the camera on the body, pixel-snapped and
  clamped to the map:

```rust
ctx.cam_target.x = self.clamp_axis(body.x.round(), self.map_w, self.view_w);
ctx.cam_target.y = self.clamp_axis(body.y.round(), self.map_h, self.view_h);
```

`clamp_axis` (`:32-39`) clamps a center so the half-view never leaves the map:
`value.clamp(view_size/2, map_size - view_size/2)`. With a `2000 x 512` map and a
`576 x 324` view this keeps `cam_target` in `[288, 1712] x [162, 350]`, so the
view never shows anything beyond the map.

The map must be drawn only for the visible area. `MapDrawSystem`
(`src/scene/battle_test/sys_map_draw.rs`) builds a world-space view rect centered
on `cam_target`, then culls and draws sections. It is called directly in
`BattleTestScene::draw` *before* `agg.draw(ctx)` so the map renders underneath
entities (`src/scene/battle_test/scene.rs:175-180`):

```rust
let view = Rect::new(
    ctx.cam_target.x - self.view_w * 0.5,
    ctx.cam_target.y - self.view_h * 0.5,
    self.view_w,
    self.view_h,
);
for section in self.map.get_sections(view) { /* draw each Some(tile) */ }
```

### `asset_preview` (static camera)

The asset-preview scene does not move the camera; it pins the values at the top of
`update` (`src/scene/asset_preview/scene.rs:226-227`):

```rust
ctx.cam_zoom = 1.0;
ctx.cam_target = vec2(self.cfg.v_width * 0.5, self.cfg.v_height * 0.5);
```

This centers the camera on `(288, 162)`, so the view shows world
`[0, 576] x [0, 324]` — the same visible region the old fixed camera produced.

## Why the math lines up

A world point `p` maps to render-target pixel
`(p.x - target.x + 576, p.y - target.y + 324)` and the blit samples the central
`576 x 324` rect (`[288, 864] x [162, 486]` in render-target pixels). That rect
corresponds to world `[target.x - 288, target.x + 288] x [target.y - 162, target.y + 162]`,
i.e. the view is centered on `cam_target`. The `flip_y: true` in the blit corrects
the render target's vertical orientation when drawing to the screen; it is
independent of the camera `target`.

## Zoom and pan clamping

The scene camera now moves to follow the target, so the render target never runs
out of buffer (the capture window is always `±576 x ±324` around the target, which
is at least the overscan margin). The remaining clamp is the map-edge clamp in
`CameraSystem` (`sys_camera.rs:32-39`), which keeps the view inside the map. If a
scene needs zoom later, it should scale the camera zoom (or `cam_zoom`) and adjust
the map-edge clamp accordingly.

## Key code locations

- `src/lib.rs` — `Context { dt, cam_zoom, cam_target }` + defaults.
- `src/util/config.rs` — `v_width`, `v_height`, `rt_overscan`, `rt_width()/rt_height()`.
- `src/util/camera.rs` — `GameRenderTarget` + `GameCamera` (`camera_at`, centered `display_rect`).
- `src/manager/mod.rs` — `begin_scene_pass` (moves camera), `blit_target` (fixed central blit), `draw_ui`, pass ordering.
- `src/scene/battle_test/sys_camera.rs` — follow camera (`cam_target` from knight, map-edge clamped).
- `src/scene/battle_test/sys_map_draw.rs` — visible-rect derivation + section culling.
- `src/scene/battle_test/scene.rs` — wires `CameraSystem` into `agg`; calls `map_draw` before `agg.draw`.
- `src/scene/asset_preview/scene.rs:226-227` — static camera (`cam_target = view center`).

## Related docs

- `docs/pixel-perfect-rendering.md` — the jitter-free render-target pipeline.
- `docs/camera-scaling.md` — macroquad `Camera2D` zoom/NDC semantics and `from_display_rect`.
- `crates/tiled/README.md` — `TiledMap` and `get_sections` section culling.
