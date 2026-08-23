# Drawing Graphics in macroquad

Macroquad draws in immediate mode: you call a `draw_*` function every frame
inside the game loop, between the frame start and `next_frame().await`. All the
functions below are available via `use macroquad::prelude::*`.

## Colors and Alpha

Colors are `Color { r, g, b, a }`, four `f32` components in the range `0.0..=1.0`.
The fourth component `a` is alpha: `0.0` is fully transparent, `1.0` is fully
opaque. You can also use the named constants (`RED`, `GREEN`, `BLUE`, `BLACK`,
`WHITE`, etc.).

```rust
let red = Color::new(1.0, 0.0, 0.0, 1.0);       // opaque red
let faint_red = Color::new(1.0, 0.0, 0.0, 0.3); // 30% opacity
let from_bytes = Color::from_rgba(255, 0, 0, 128); // 0-255 form
let from_hex = Color::from_hex(0xFF0000);          // alpha defaults to 1.0
let transparent_blue = BLUE.with_alpha(0.5);       // copy with new alpha
```

To blend a color into the background, set the alpha on the color you pass to the
draw call; macroquad applies standard alpha blending.

## Solid Shapes

```rust
draw_rectangle(x, y, w, h, color);         // filled, top-left at (x, y)
draw_circle(x, y, r, color);               // filled, centered at (x, y)
draw_triangle(v1, v2, v3, color);          // filled, three Vec2 vertices
draw_ellipse(x, y, w, h, rotation, color); // filled, rotation in degrees
draw_poly(x, y, sides, radius, rotation, color); // filled regular polygon
draw_line(x1, y1, x2, y2, thickness, color);    // segment with thickness
```

## Outlines

Every solid shape has a `_lines` counterpart that draws only the outline, with
an extra `thickness` argument:

```rust
draw_rectangle_lines(x, y, w, h, thickness, color);
draw_circle_lines(x, y, r, thickness, color);
draw_triangle_lines(v1, v2, v3, thickness, color);
draw_ellipse_lines(x, y, w, h, rotation, thickness, color);
draw_poly_lines(x, y, sides, radius, rotation, thickness, color);
```

To draw a shape with both a fill and a border, call the solid version then the
`_lines` version on top.

## Rotation and Origin (extended rectangle)

`draw_rectangle_ex` accepts a `DrawRectangleParams` for rotation and a custom
pivot point:

```rust
draw_rectangle_ex(
    x, y, w, h,
    DrawRectangleParams {
        offset: vec2(0.5, 0.5),  // rotate around the rectangle center
        rotation: 45.0,          // degrees
        color,
    },
);
```

## Minimal Example

```rust
use macroquad::prelude::*;

#[macroquad::main("Graphics")]
async fn main() {
    loop {
        clear_background(DARKGRAY);

        draw_rectangle(20.0, 20.0, 100.0, 60.0, Color::new(0.2, 0.6, 0.2, 0.8));
        draw_rectangle_lines(20.0, 20.0, 100.0, 60.0, 2.0, GREEN);
        draw_circle(300.0, 100.0, 30.0, RED.with_alpha(0.5));
        draw_line(20.0, 120.0, 300.0, 120.0, 4.0, BLUE);

        next_frame().await;
    }
}
```
