# Pixel-Perfect Rendering Without Shaders

This document explains how the game achieves jitter-free, pixel-perfect pixel-art
rendering using render targets and nearest-neighbor filtering, without needing
custom shaders.

## The Problem: Pixel-Art Jitter

Classic pixel-art jitter shows up when a camera moves or zooms by fractional
amounts. A sprite at world position `(100.5, 50.5)` maps to a fractional pixel on
screen, so:

- With linear filtering, the GPU blends neighboring texels, making edges fuzzy
  and "swimming" as the camera moves.
- With nearest filtering, the sprite snaps to whole texels, but at non-integer
  screen positions it steps unevenly, producing shimmer.

The usual fixes are shaders that snap geometry to whole pixels
(`pixel_snap.vert`) or do texel-correct sampling (`jitter_free.frag`). This
project originally used both, but they turned out to be redundant: the render
pipeline already prevents fractional-pixel rasterization in the first place.

## The Pipeline

The game never draws the scene directly to the screen. Instead it:

1. Renders the scene into an offscreen render target at an **integer scale**.
2. Blits that target to the screen with **nearest-neighbor filtering**.
3. Follows the scene by re-centering the scene `Camera2D` on a world-space
   target and blitting a **fixed central sub-rectangle** of the target (rather
   than panning the sample rect).

### 1. Integer (2x) overscan render target

`src/util/config.rs` defines a logical view and an overscan factor:

```rust
v_width: 576.0,   // 640.0 * 0.9
v_height: 324.0,  // 360.0 * 0.9
rt_overscan: 2.0,
```

The render target is `rt_overscan` times the logical view
(`src/util/camera.rs`):

```rust
let target = render_target(w, h); // 1152 x 648
target.texture.set_filter(FilterMode::Nearest);
```

Because the scale factor is exactly 2, any world coordinate maps to an integer
target texel. A fractional world position like `(100.5, 50.5)` lands on whole
pixel `(201, 101)`. Sprites are therefore never rasterized at a sub-pixel
boundary, which removes the source of jitter entirely.

The camera's `display_rect` is centered so the logical 576x324 view sits in the
middle of the 1152x648 buffer:

```rust
let display_rect = Rect::new(
    -(rt_w - config.v_width) * 0.5,
    -(rt_h - config.v_height) * 0.5,
    rt_w,
    rt_h,
);
```

### 2. Nearest-neighbor filtering

The render target texture is created with `FilterMode::Nearest`
(`src/util/camera.rs`). When the target is later sampled, no linear blending
happens between texels, so edges stay hard and crisp even when the final screen
scale is not an integer.

### 3. Camera follow via a fixed central blit

The scene `Camera2D` is re-centered on a world-space target each frame, and the
blit always samples the central sub-rectangle of the buffer. The scene sets
`Context.cam_target` to the point to follow (`src/scene/battle_test/sys_camera.rs`):

```rust
ctx.cam_target.x = clamp_axis(body.x.round(), map_w, view_w); // follow the knight
```

`Manager::begin_scene_pass` builds the camera centered on `cam_target`
(`src/manager/mod.rs`), and `blit_target` blits the fixed central rect:

```rust
let src = Rect::new(
    (self.cfg.rt_width() - self.cfg.v_width) * 0.5,
    (self.cfg.rt_height() - self.cfg.v_height) * 0.5,
    self.cfg.v_width,
    self.cfg.v_height,
);

draw_texture_ex(&self.camera.render_target.texture, offset.x, offset.y, WHITE,
    DrawTextureParams {
        dest_size: Some(vec2(self.cfg.v_width * scale, self.cfg.v_height * scale)),
        source: Some(src),
        flip_y: true,
        ..Default::default()
    });
```

Because the camera target (and sprite positions) are rounded to whole pixels, and
the blit rect is fixed, sprites are rasterized at whole render-target texels,
giving stable, jitter-free motion.

## The Role of the Overscan Margin

The buffer is larger than the visible view (1152x648 vs 576x324), leaving extra
rendered pixels around the edge. Because the camera is centered on the target, the
buffer always captures `±576 x ±324` around it — far more than the visible view —
so the fixed central `source` rect never runs off the rendered image. The only
clamp needed is the map-edge clamp in `CameraSystem`, which keeps the view inside
the map.

## Why the Shaders Were Redundant

Two shaders were originally applied on top of this pipeline:

- `pixel_snap.vert` snaps quad vertices to whole screen pixels (fixes geometry
  jitter when sprites land on fractional pixels).
- `jitter_free.frag` does analytic texel-correct bilinear sampling (fixes
  texture sampling jitter).

Both solve the *direct-camera* case, where sprites land on fractional pixels.
The render-target pipeline prevents fractional-pixel rasterization in the first
place, so the shaders have nothing left to correct. They can be omitted
entirely (the `gl_use_material` calls in `Manager::draw_scene` / `blit_target`
are commented out) with no visual difference.

## Key Code Locations

- `src/util/config.rs` — `rt_overscan`, `rt_width`/`rt_height`
- `src/util/camera.rs` — render target creation, `FilterMode::Nearest`,
  centered `display_rect`, `camera_at(target)`
- `src/manager/mod.rs` — `begin_scene_pass` (moves the camera), `blit_target` (fixed central blit)
- `src/scene/battle_test/sys_camera.rs` — `cam_target` follow + map-edge clamp
- `src/lib.rs` — `Context { cam_zoom, cam_target }`
