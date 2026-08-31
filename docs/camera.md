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

`Manager::draw` (`src/manager/mod.rs`) runs four passes in order:

1. `begin_scene_pass(ctx)` — builds a `Camera2D` centered on `ctx.cam_target`
   via `camera.camera_at(...)`, `set_camera(...)`, then `clear_background(BLACK)`.
2. `draw_scene(ctx)` — applies the **pixel-snap** material (if loaded and
   `Config.pixel_snap` is on), then calls `scene.draw(ctx)`, then restores the
   default material.
3. `begin_screen_pass()` — `set_default_camera()`, clear.
4. `blit_target()` — applies the **jitter-free** material, draws the render
   target to the screen, restores the default material, then `draw_ui(ctx)`.

See `docs/pixel-perfect-rendering.md` for the two shaders.

### `blit_target`: a fixed central view

`blit_target` samples the fixed central sub-rectangle of the render target (the
logical view) and scales it to fill the window using `view_scale::view_scale()`
(`src/util/view_scale.rs`):

```rust
let src = Rect::new(
    (rt_w - v_width) * 0.5,
    (rt_h - v_height) * 0.5,
    v_width,
    v_height,
);
draw_texture_ex(&rt.texture, offset.x, offset.y, WHITE,
    DrawTextureParams { dest_size: ..., source: Some(src), flip_y: true, ..Default::default() });
```

The source rect is always the central `640 x 360` sub-rectangle of the
`1280 x 720` buffer, so it never runs off the render target. The visible world
is chosen by moving the scene camera, not by shifting this rect. The screen
scale is `min(screen_width()/640, screen_height()/360)` — non-integer in
general — so the jitter-free material is applied to avoid moiré (again, see
`docs/pixel-perfect-rendering.md`).

### The UI pass has its own camera

`draw_ui` builds a `Camera2D` on the fly with `zoom = vec2(2.0 / v_width, 2.0 / v_height)`
and a viewport derived from the same `view_scale` result as `blit_target`, then
draws `scene.draw_ui(ctx)`. UI rendering is unaffected by `cam_target`, and the
shared scale/offset keeps UI aligned with the blitted scene.

## `GameCamera` setup

`src/util/camera.rs`:

- `GameRenderTarget` creates the render target at `rt_width x rt_height`
  (`1280 x 720`) with `FilterMode::Nearest`.
- `GameCamera` stores:
  - `display_rect = Rect::new(-(rt_w - v_width)*0.5, -(rt_h - v_height)*0.5, rt_w, rt_h)`
    — the logical view centered inside the larger buffer.
  - `render_target` (cloned from `GameRenderTarget`).
  - `camera_at(target)` builds a fresh `Camera2D` centered on `target`:
    `Camera2D::from_display_rect(display_rect)` (1:1 world-to-pixel), then sets
    `camera.target = target` and `camera.render_target = Some(render_target)`.

Because `from_display_rect` maps 1 world unit to 1 render-target pixel, moving
the camera `target` shifts the capture window one render-target pixel per world
unit. A world point `p` maps to render-target pixel
`(p.x - target.x + rt_w/2, p.y - target.y + rt_h/2)`, so the buffer always
captures the world rectangle
`[target.x - 640, target.x + 640] x [target.y - 360, target.y + 360]`.

## Config

`src/util/config.rs`:

- `v_width = 640.0`, `v_height = 360.0`.
- `rt_overscan = 2.0`; `rt_width() = 1280.0`, `rt_height() = 720.0`.
- Window `1280x720`.
- `pixel_snap: bool` (default `true`) gates the pixel-snap vertex shader.

## `Context`

`src/lib.rs`:

```rust
pub struct Context { pub dt: f32, pub cam_zoom: f32, pub cam_target: Vec2 }
```

Defaults set in `init()`: `dt: 0.0`, `cam_zoom: 1.0`, `cam_target: Vec2::ZERO`.
`dt` is set by `Manager::update` from `get_frame_time()`; systems read `ctx.dt`
rather than calling `get_frame_time()` themselves.

## Driving the camera from a scene

### `battle_test` (follow the hero)

The scene follows a small marker entity, the `Orb`, which `CameraOrbSystem`
(`src/scene/battle_test/sys_orb.rs`) places 25px ahead of the knight's center,
offset in the direction it faces.

The follow camera is a `CameraSystem` registered in `BattleTestScene::load`
(`src/scene/battle_test/scene.rs`):

```rust
self.agg.add_update(CameraSystem::new(store, cfg.v_width, cfg.v_height, map_w, map_h));
```

`CameraSystem` (`src/scene/battle_test/sys_camera.rs`):

- `locate_orb()` finds the `Orb` entity and reads its `pos`.
- `update(ctx)` clamps the orb position to the map, then eases the smoothed
  camera position toward it with a critically damped spring
  (`spring_toward`, `CAMERA_FREQUENCY = 5.0` — velocity-tracked, closed-form,
  overshoot-free and frame-rate independent):

```rust
let target = vec2(
    self.clamp_axis(orb.pos.x, self.map_w, self.view_w),
    self.clamp_axis(orb.pos.y, self.map_h, self.view_h),
);
let (pos, vel) = spring_toward(self.smooth, self.velocity, target, ctx.dt);
self.smooth = pos;
self.velocity = vel;
ctx.cam_target = self.smooth;
```

`clamp_axis` clamps a center so the half-view never leaves the map:
`value.clamp(view_size/2, map_size - view_size/2)` (with a fallback to
`map_size/2` when the map is smaller than the view). The target is deliberately
**not** rounded — `cam_target` stays sub-pixel, and the pixel-snap vertex shader
handles whole-texel snapping during the scene pass.

The map must be drawn only for the visible area. `MapDrawSystem`
(`src/scene/battle_test/sys_map_draw.rs`) builds a world-space view rect centered
on `cam_target`, then culls and draws sections. It is registered as a draw
system in `BattleTestScene::load` before the entity `DrawSystem`, so the map
renders underneath entities.

### `asset_preview` (static camera)

The asset-preview scene does not move the camera; it pins the values at the top
of `update` (`src/scene/asset_preview/scene.rs`):

```rust
ctx.cam_zoom = 1.0;
ctx.cam_target = vec2(self.cfg.v_width * 0.5, self.cfg.v_height * 0.5);
```

This centers the camera on `(320, 180)`, so the view shows world
`[0, 640] x [0, 360]`.

## Why the math lines up

A world point `p` maps to render-target pixel
`(p.x - target.x + 640, p.y - target.y + 360)` and the blit samples the central
`640 x 360` rect (`[320, 960] x [180, 540]` in render-target pixels). That rect
corresponds to world
`[target.x - 320, target.x + 320] x [target.y - 180, target.y + 180]`, i.e. the
view is centered on `cam_target`. The `flip_y: true` in the blit corrects the
render target's vertical orientation when drawing to the screen; it is
independent of the camera `target`.

## Zoom and pan clamping

The scene camera moves to follow the target, so the render target never runs out
of buffer (the capture window is always `±640 x ±360` around the target, which is
at least the overscan margin). The remaining clamp is the map-edge clamp in
`CameraSystem` (`sys_camera.rs`), which keeps the view inside the map. If a
scene needs zoom later, it should scale the camera zoom (or `cam_zoom`) and
adjust the map-edge clamp accordingly.

## Key code locations

- `src/lib.rs` — `Context { dt, cam_zoom, cam_target }` + defaults.
- `src/util/config.rs` — `v_width`, `v_height`, `rt_overscan`, `rt_width()/rt_height()`, `pixel_snap`.
- `src/util/camera.rs` — `GameRenderTarget` + `GameCamera` (`camera_at`, centered `display_rect`).
- `src/util/view_scale.rs` — shared fill-the-window scale/offset helper.
- `src/manager/mod.rs` — `begin_scene_pass`, `draw_scene` (pixel-snap), `blit_target` (jitter-free), `draw_ui`, pass ordering.
- `src/scene/battle_test/sys_orb.rs` — the `Orb` marker the camera follows.
- `src/scene/battle_test/sys_camera.rs` — follow camera (smoothed `cam_target`, map-edge clamped).
- `src/scene/battle_test/sys_map_draw.rs` — visible-rect derivation + section culling.
- `src/scene/battle_test/scene.rs` — wires `CameraSystem` into `agg` and registers draw systems.
- `src/scene/asset_preview/scene.rs` — static camera (`cam_target = view center`).

## Related docs

- `docs/pixel-perfect-rendering.md` — the two shaders and the jitter-free pipeline.
- `docs/camera-scaling.md` — macroquad `Camera2D` zoom/NDC semantics and `from_display_rect`.
- `crates/tiled/README.md` — `TiledMap` and `get_sections` section culling.
