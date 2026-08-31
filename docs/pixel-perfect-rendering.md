# Pixel-Perfect Rendering

This document explains how the game renders pixel-art crisply at any window
size, using two custom shaders: `pixel_snap.vert` (geometry snapping) and
`jitter_free.frag` (texel-correct sampling).

## The Problem

Pixel art breaks in two ways when a camera moves by fractional amounts or the
final image is scaled by a non-integer factor:

- **Geometry jitter.** A sprite whose vertices land on fractional pixels is
  rasterized unevenly, so it "shimmers" or "swims" as the camera pans.
- **Texture moiré.** When a low-res image is upscaled by a non-integer factor
  with nearest-neighbor sampling, each source texel maps to a different number
  of screen pixels (e.g. 2 then 3), producing crawling "lines of waves" that
  shift as the content scrolls.

The project fixes both with a two-stage pipeline plus one shader per stage.

## The Pipeline

The scene is never drawn directly to the screen. `Manager::draw`
(`src/manager/mod.rs`) runs four passes:

1. `begin_scene_pass(ctx)` — sets the scene `Camera2D` centered on
   `ctx.cam_target` and clears the render target.
2. `draw_scene(ctx)` — draws the scene with the **pixel-snap** material applied
   (geometry snapped to whole render-target texels).
3. `begin_screen_pass()` — switches to the default screen camera and clears.
4. `blit_target()` — draws the render target to the screen with the
   **jitter-free** material applied (texel-correct sampling), then `draw_ui(ctx)`.

### Stage 1: 2x overscan render target + `pixel_snap.vert`

`src/util/config.rs` defines a 640x360 logical view with a 2x overscan factor:

```rust
v_width: 640.0,
v_height: 360.0,
rt_overscan: 2.0,   // rt_width()/rt_height() = 1280 x 720
```

The render target is 1280x720 (`src/util/camera.rs`) with
`FilterMode::Nearest`. `GameCamera::camera_at(target)` uses
`Camera2D::from_display_rect`, so 1 world unit maps to exactly 1 render-target
texel.

During the scene pass, `draw_scene` applies `pixel_snap.vert`
(`assets/shaders/pixel_snap.vert`). This vertex shader converts each vertex to
its render-target pixel coordinate, rounds it to the nearest whole pixel
(`floor(b + 0.5)`), and converts back:

```glsl
vec4 pos = Projection * Model * vec4(position, 1);
vec2 ndc = pos.xy / pos.w;
vec2 b   = ndc * half_vp + half_vp;   // pixel coords
b        = floor(b + 0.5);            // snap to whole texel
...
```

The `viewport` uniform is set to the render-target size (1280x720). Because
every vertex is snapped to a whole texel, sprites are never rasterized at
sub-texel boundaries — no geometry jitter. The scene camera is therefore free
to move in sub-pixel increments; no CPU-side `.round()` calls are needed.

The material is loaded in `Manager::new` (failure-tolerant) and is gated by the
`Config.pixel_snap` flag (default `true`). It uses macroquad's default alpha
blending so translucent sprites composite correctly.

### Stage 2: fill-the-window blit + `jitter_free.frag`

`blit_target` draws the central 640x360 sub-rectangle of the render target to
the screen, scaled to fill the window while preserving aspect ratio. The scale
and centering offset come from a single shared helper,
`view_scale::view_scale()` (`src/util/view_scale.rs`):

```rust
let scale = f32::min(screen_width() / v_width, screen_height() / v_height);
let offset = ((screen - v * scale) * 0.5);
```

This scale is only an integer by accident. At non-integer scales, plain
nearest-neighbor sampling would produce the crawling moiré described above.
`blit_target` therefore applies `jitter_free.frag`
(`assets/shaders/jitter_free.frag`), a texel-correct "sharp-bilinear" sampler:

```glsl
vec2 texel = 1.0 / texture_size;
vec2 p     = uv * texture_size;      // position in texel space
vec2 ddxy  = max(fwidth(p), 1e-6);   // texels per screen pixel
vec2 k     = 1.0 / ddxy;             // screen pixels per texel
// ... blend the 4 surrounding texels with a sub-pixel-wide ramp
```

For each screen pixel it computes its footprint in render-target texel space
(via `fwidth`) and blends only across the sub-pixel boundary band. At integer
scales the ramp is a hard edge (crisp); at fractional scales it produces a
single-screen-pixel anti-aliased transition instead of moiré. This lets the
image fill the window at *any* size — growing and shrinking with the window —
without the "lines of waves".

The `texture_size` uniform is set to the full render-target size (1280x720),
because `draw_texture_ex`'s `source` rect produces UVs in full-texture space.
The render target keeps `FilterMode::Nearest`; the shader does its own point
fetches at texel centers, so no linear filtering is involved. The material is
loaded in `Manager::new` (failure-tolerant): if the shader sources are missing
or fail to compile, the blit falls back to plain nearest sampling.

The UI pass shares the same `view_scale` result: `draw_ui` builds its camera
viewport from it, and `UI::begin` (`src/ui/core.rs`) maps the mouse position
with the same scale/offset so hit-testing stays aligned.

## Why the two shaders are complementary

`pixel_snap.vert` guarantees the render-target content is texel-aligned; that is
the precondition `jitter_free.frag` needs to sample it correctly. Neither alone
suffices: geometry snapping without texel-correct blitting still moirés at
non-integer window scales, and texel-correct sampling without aligned content
just anti-aliases sub-pixel geometry into softness.

## Key Code Locations

- `src/util/config.rs` — `v_width`, `v_height`, `rt_overscan`, `rt_width()/rt_height()`, `pixel_snap` flag.
- `src/util/camera.rs` — render target creation (`FilterMode::Nearest`), `GameCamera::camera_at`.
- `src/util/view_scale.rs` — shared fill-the-window scale/offset helper.
- `src/manager/mod.rs` — `draw_scene` (pixel-snap material), `blit_target` (jitter-free material), `draw_ui`, `load_shader_material`.
- `assets/shaders/pixel_snap.vert` — vertex snapping (geometry jitter).
- `assets/shaders/jitter_free.frag` — texel-correct sampling (texture moiré).
- `src/ui/core.rs` — `UI::begin` mouse mapping (same `view_scale`).
