# Writing Shaders in macroquad

A shader is a small program that runs on the GPU. In macroquad (via
[`miniquad`](https://docs.rs/miniquad/latest/miniquad/)) shaders are written in
GLSL (`#version 100`), a C-like language. A shader has two parts:

- **Vertex shader** — runs once per vertex, transforms 3D positions into screen
  coordinates (`gl_Position`).
- **Fragment shader** — runs once per pixel, sets `gl_FragColor` to the color of
  that pixel.

For a 2D game the vertex shader is usually a fixed boilerplate that just
transforms the position; all the interesting work happens in the fragment
shader.

Everything below is available via `use macroquad::prelude::*` (which re-exports
the `material` module and miniquad's shader types such as `ShaderSource`,
`MaterialParams`, `UniformDesc`, and `UniformType`).

## The material API

Macroquad's shader entry point is a `Material`, created once with `load_material`
and then activated around draw calls with `gl_use_material`.

```rust
let material = load_material(
    ShaderSource::Glsl {
        vertex: VERTEX_SHADER,
        fragment: FRAGMENT_SHADER,
    },
    MaterialParams {
        uniforms: vec![UniformDesc::new("my_uniform", UniformType::Float1)],
        pipeline_params: PipelineParams::default(),
        textures: vec![],
    },
)
.unwrap();

// ...inside the game loop...
gl_use_material(&material);
draw_rectangle(0.0, 0.0, 100.0, 100.0, WHITE); // drawn with this material
gl_use_default_material();                       // back to the normal material
```

Key items:

- `load_material(source, params) -> Result<Material, ShaderError>`. It can fail
  (e.g. a GLSL compile error), so don't ignore the `Result` in a real game.
- `gl_use_material(&material)` — every subsequent `draw_*` call uses this
  material until you switch back.
- `gl_use_default_material()` — restores macroquad's built-in material.
- `MaterialParams` is used once at load time; it cannot be changed afterwards.
- `Material` has three runtime setters:
  - `set_uniform(name, value)` — set a uniform (see below).
  - `set_uniform_array(name, &[values])` — set an array uniform.
  - `set_texture(name, texture)` — bind an extra `Texture2D` by name.

## Built-in uniforms and vertex attributes

Macroquad automatically injects these uniforms into every shader:

| Uniform        | Type        | Meaning                                        |
| -------------- | ----------- | ---------------------------------------------- |
| `_Time`        | `vec4`      | `_Time.x` is seconds since the game started    |
| `Model`        | `mat4`      | Model transform matrix                         |
| `Projection`   | `mat4`      | Projection matrix                              |
| `Texture`      | `sampler2D` | The texture bound to the current draw call     |
| `_ScreenTexture` | `sampler2D` | Snapshot of the current render target (special) |

The vertex shader receives these vertex attributes:

| Attribute | Type   | Notes                                        |
| --------- | ------ | -------------------------------------------- |
| `position` | `vec3` | Vertex position in world space               |
| `texcoord` | `vec2` | Texture UV coordinates                       |
| `color0`   | `vec4` | Per-vertex color, components in `0.0..=255.0` |

Note that `color0` is in the `0..255` range, so divide by `255.0` to get a
`0..1` color in the fragment shader.

## A minimal custom material

The standard 2D vertex shader, then a fragment shader that tints the drawn
texture by a uniform color:

```rust
const VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;

varying lowp vec2 uv;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = texcoord;
}"#;

const FRAGMENT: &str = r#"#version 100
varying lowp vec2 uv;

uniform sampler2D Texture;
uniform lowp vec4 tint;

void main() {
    gl_FragColor = tint * texture2D(Texture, uv);
}"#;
```

```rust
let material = load_material(
    ShaderSource::Glsl { vertex: VERTEX, fragment: FRAGMENT },
    MaterialParams {
        uniforms: vec![UniformDesc::new("tint", UniformType::Float4)],
        ..Default::default()
    },
)
.unwrap();

loop {
    clear_background(GRAY);

    material.set_uniform("tint", vec4(1.0, 0.0, 0.0, 1.0));
    gl_use_material(&material);
    draw_texture(&my_texture, 0.0, 0.0, WHITE);
    gl_use_default_material();

    next_frame().await;
}
```

## Uniforms

Declare each custom uniform in `MaterialParams::uniforms` at load time, using
`UniformDesc::new(name, UniformType)`. `UniformType` has these variants:

- `Float1`, `Float2`, `Float3`, `Float4` — `f32` vectors.
- `Int1`, `Int2`, `Int3`, `Int4` — `u32` vectors.
- `Mat4` — 4x4 float matrix.

Arrays are declared with `UniformDesc::array(desc, len)`:

```rust
uniforms: vec![UniformDesc::array(
    UniformDesc::new("colors", UniformType::Float4),
    10,
)],
```

At runtime, `set_uniform` is generic and accepts tuples and glam types matching
the declared `UniformType`:

```rust
material.set_uniform("speed", 0.5);                       // Float1
material.set_uniform("size", (320.0, 180.0));             // Float2
material.set_uniform("position", (x, y, z));              // Float3
material.set_uniform("tint", vec4(1.0, 0.0, 0.0, 1.0));   // Float4
```

A uniform whose name is not in the `uniforms` list is silently ignored, so keep
the load-time names and runtime names in sync.

Arrays use `set_uniform_array`:

```rust
let colors: [Vec4; 10] = [vec4(0.0, 0.0, 0.0, 1.0); 10];
material.set_uniform_array("colors", &colors[..]);
```

## Post-processing / fullscreen passes

The most common shader use case is a post-processing effect over the whole
screen. You render the scene into a `RenderTarget`, then draw that texture back
to the screen with a custom material.

```rust
let render_target = render_target(320, 150);
render_target.texture.set_filter(FilterMode::Nearest);

let material = load_material(
    ShaderSource::Glsl { vertex: VERTEX, fragment: CRT_FRAGMENT },
    Default::default(),
)
.unwrap();

loop {
    // 1. draw the scene into the render target
    set_camera(&Camera2D {
        zoom: vec2(0.01, 0.01),
        target: vec2(0.0, 0.0),
        render_target: Some(render_target.clone()),
        ..Default::default()
    });
    clear_background(LIGHTGRAY);
    // ...draw the game...

    // 2. draw the rendered scene back to the screen through the shader
    set_default_camera();
    clear_background(WHITE);
    gl_use_material(&material);
    draw_texture_ex(
        &render_target.texture,
        0.0, 0.0, WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(screen_width(), screen_height())),
            ..Default::default()
        },
    );
    gl_use_default_material();

    next_frame().await;
}
```

The same pattern drives fullscreen "Shadertoy-style" effects (e.g. a starfield)
where the fragment shader ignores `Texture` entirely and computes each pixel
from `gl_FragCoord` and a resolution uniform.

## Screen-reading shaders (`_ScreenTexture`)

Some effects (water refraction, glass, blur) need to read the pixels already
drawn this frame. Declare `_ScreenTexture` in the shader and macroquad handles
the rest:

```glsl
uniform sampler2D _ScreenTexture;
uniform vec4 _Time;
```

When a material using `_ScreenTexture` is used for the first time in a frame,
macroquad snapshots the current render target into `_ScreenTexture`, so the
fragment shader can sample the scene as it has been drawn so far.

## Cross-platform: GLSL vs Metal

On macOS/iOS the backend is Metal, which needs a Metal Shading Language (MSL)
program instead of GLSL. Provide both and pick based on the active backend:

```rust
use macroquad::window::miniquad::*;

let ctx = unsafe { get_internal_gl().quad_context };
let material = load_material(
    match ctx.info().backend {
        Backend::OpenGl => ShaderSource::Glsl { vertex: VERTEX, fragment: FRAGMENT },
        Backend::Metal  => ShaderSource::Msl  { program: METAL },
    },
    MaterialParams {
        uniforms: vec![UniformDesc::new("tint", UniformType::Float4)],
        ..Default::default()
    },
)
.unwrap();
```

`get_internal_gl()` is `unsafe`. In the MSL program, uniforms must be laid out
with `Model`, `Projection`, `_Time` in that exact order at the start of the
uniform struct (they share a single buffer), followed by any custom uniforms.

## Gotchas

- Use `#version 100` (WebGL1/GLES2-style) GLSL. `texture2D(...)` instead of
  `texture(...)`, and `gl_FragColor` instead of `out` variables.
- `color0` is `0..255`; divide by `255.0` before using as a color.
- A `set_uniform` for a name not declared at load time is silently ignored.
- `load_material` compiles the shader and returns `Result`; on a compile error
  you get a `ShaderError`, which is useful for iterating on shader code.
- Keep `gl_use_material` / `gl_use_default_material` balanced: any draw call
  between them uses the custom material.

## Further resources

- macroquad `material` module docs:
  https://docs.rs/macroquad/latest/macroquad/material/index.html
- miniquad `graphics` module (`ShaderSource`, `UniformType`, `UniformDesc`,
  `PipelineParams`): https://docs.rs/miniquad/latest/miniquad/graphics/index.html
- Official examples:
  - `custom_material.rs` — uniforms, uniform arrays, blend params, Metal vs GLSL:
    https://github.com/not-fl3/macroquad/blob/master/examples/custom_material.rs
  - `post_processing.rs` — render-to-texture + CRT shader:
    https://github.com/not-fl3/macroquad/blob/master/examples/post_processing.rs
  - `shadertoy.rs` — editable shader playground:
    https://github.com/not-fl3/macroquad/blob/master/examples/shadertoy.rs
- "Game Development in Rust with Macroquad" — starfield shader chapter:
  https://mq.agical.se/ch9-starfield-shader.html
- "Platformer book" — post-processing and screen-reading shaders:
  https://not-fl3.github.io/platformer-book/post-effects.html
  https://not-fl3.github.io/platformer-book/screen-reading.html
