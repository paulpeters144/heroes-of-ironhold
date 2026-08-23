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
3. Applies zoom/pan as a **sub-rectangle sample** of the target, not by moving
   the camera.

### 1. Integer (2x) overscan render target

`src/util/config.rs` defines a logical view and an overscan factor:

```rust
v_width: 320.0,
v_height: 180.0,
rt_overscan: 2.0,
```

The render target is `rt_overscan` times the logical view
(`src/util/camera.rs`):

```rust
let target = render_target(w, h); // 640 x 360
target.texture.set_filter(FilterMode::Nearest);
```

Because the scale factor is exactly 2, any world coordinate maps to an integer
target texel. A fractional world position like `(100.5, 50.5)` lands on whole
pixel `(201, 101)`. Sprites are therefore never rasterized at a sub-pixel
boundary, which removes the source of jitter entirely.

The camera's `display_rect` is centered so the logical 320x180 view sits in the
middle of the 640x360 buffer:

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

### 3. Zoom and pan as a sub-rect blit

The scene camera itself never moves. `cam_zoom` and `cam_pan` are just numbers
on `Context`, mutated by input (`src/scene/opening/scene.rs`):

```rust
ctx.cam_zoom *= 1.0 + ZOOM_SPEED * dt; // Q / E keys
ctx.cam_pan.x += step;                  // arrow keys
```

They are only consumed in `Manager::blit_target` (`src/manager/mod.rs`), which
computes a `source` rectangle into the already-rendered 2x buffer and draws it
with `draw_texture_ex`:

```rust
let view = vec2(self.cfg.v_width / ctx.cam_zoom, self.cfg.v_height / ctx.cam_zoom);
let src = Rect::new(
    self.cfg.rt_width() * 0.5 + ctx.cam_pan.x - view.x * 0.5,
    self.cfg.rt_height() * 0.5 + ctx.cam_pan.y - view.y * 0.5,
    view.x,
    view.y,
);

draw_texture_ex(&self.camera.render_target.texture, offset.x, offset.y, WHITE,
    DrawTextureParams {
        dest_size: Some(vec2(self.cfg.v_width * scale, self.cfg.v_height * scale)),
        source: Some(src),
        flip_y: true,
        ..Default::default()
    });
```

Since the pan samples a Nearest-filtered texture, it effectively snaps to whole
texels (half-world-pixel steps at 2x), giving stable, jitter-free motion.

## The Role of the Overscan Margin

The buffer is larger than the visible view (640x360 vs 320x180), leaving extra
rendered pixels around the edge. That margin is what lets the `source` rect roam
during pan and zoom without running off the rendered image.

The min zoom is clamped so the view never grows past that margin
(`src/scene/opening/scene.rs`):

```rust
ctx.cam_zoom = ctx.cam_zoom.clamp(1.0 / self.cfg.rt_overscan, 8.0);
```

`cam_pan` is likewise clamped so the source rect stays inside the buffer.

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
  centered `display_rect`
- `src/manager/mod.rs` — `blit_target` sub-rect blit
- `src/scene/opening/scene.rs` — `cam_zoom`/`cam_pan` input and clamping
- `src/lib.rs` — `Context { cam_zoom, cam_pan }`
