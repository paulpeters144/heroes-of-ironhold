# Research Topic
How to draw graphics in macroquad, using the knight in `src/scene/asset_preview/` as the working example — specifically how the knight's two attack frames (a thrust and a swipe) are represented, selected, and drawn.

## How macroquad draws graphics (immediate mode)

Macroquad draws in immediate mode: every frame, inside the game loop between
frame start and `next_frame().await`, you call `draw_*` functions. There is no
retained scene graph. All drawing is done via `use macroquad::prelude::*`.

- Solid shapes: `draw_rectangle`, `draw_circle`, `draw_triangle`, `draw_ellipse`,
  `draw_poly`, `draw_line`. Each has a `_lines` outline variant that takes an
  extra `thickness`.
- Textures: `draw_texture` (whole image) and `draw_texture_ex` (with a
  `DrawTextureParams` for sub-rect sampling, rotation, flipping, resizing).
- Colors are `Color { r, g, b, a }` with components in `0.0..=1.0`.

Reference: `docs/macroquad-graphics.md`.

## The key function: `draw_texture_ex` and `DrawTextureParams`

`DrawSystem::draw_animation` (`src/systems/sys_draw.rs:24`) is the single place
the knight frames actually get blitted:

```rust
draw_texture_ex(
    &animation.source,
    animation.position.x,
    animation.position.y,
    animation.tint,
    DrawTextureParams {
        dest_size: Some(dest_size),
        source: Some(source),
        flip_x: animation.flip_x,
        ..Default::default()
    },
);
```

`source` is a `Rect` that selects one frame out of the horizontal strip:

```rust
let source = Rect::new(
    animation.current_frame as f32 * animation.frame_width,
    0.0,
    animation.frame_width,
    animation.frame_height,
);
```

So the spritesheet is a single horizontal strip of frames; the frame is chosen
purely by offsetting the source rect by `current_frame * frame_width`.

The full `DrawTextureParams` struct (from docs.rs) also supports, beyond what
this project currently uses:

```rust
pub struct DrawTextureParams {
    pub dest_size: Option<Vec2>,
    pub source: Option<Rect>,
    pub rotation: f32,      // radians
    pub flip_x: bool,
    pub flip_y: bool,
    pub pivot: Option<Vec2>, // screen-space point to rotate around; None = texture center
}
```

Notably `rotation` (radians) and `pivot` (screen-space) are available but
**currently unused** in `sys_draw.rs`. This matters for the swipe attack if the
plan wants an actual sweeping arc rather than a static swapped frame. Also note
`pivot` is in *screen space* (e.g. `pivot(0,0)` rotates around the top-left of
the screen), not texture-local coordinates.

The render pipeline is pixel-art friendly: the scene renders into an integer 2x
overscan render target with `FilterMode::Nearest`, then blits to screen
(`docs/pixel-perfect-rendering.md`).

## Sprite asset layout

All knight art lives in `assets/images/knight/` and is registered in
`src/access/ids.rs` (`images::Knight::{Knight1..Sword3}`).

| Asset | File (variant 1) | Pixel size | Frames |
|-------|------------------|------------|--------|
| Body  | `anim-knight-1.png` | 384 x 64 | 6 frames of 64x64 |
| Sword | `anim-knight-sword-1.png` | 64 x 32 | 2 frames of 32x32 |
| Shield| `anim-knight-shield-1.png` | 32 x 32 | 1 (single image) |

The body spritesheet is a 6-frame horizontal strip (384 = 6 x 64), matching
`FRAME_SIZE = 64.0` and `FRAME_COUNT = 6` in `scene.rs`. The sword sheet has two
32x32 frames matching `SWORD_FRAME_SIZE = 32.0` / `SWORD_FRAME_COUNT = 2`.

## The knight in the asset preview scene (architecture)

The scene (`src/scene/asset_preview/scene.rs`) is an ECS demo using
`pico_entity_store`. The knight is composed of three separate entities drawn as
overlapping layers (z-ordered: body 0, shield 1, sword 2):

- **Body** — a `Knight` entity with an `Animation` child (the 6-frame strip).
- **Shield** — a `Shield` entity with a `StaticImage` child (single 32x32).
- **Sword** — a `Sword` entity with an `Animation` child (2-frame strip).

Both the shield and sword carry a `Held` marker so the offset system knows they
are attached to the knight body (the right-hand-side "boxes" entities do not).

Systems registered on `SystemAgg` (in `new()`):

1. `AnimationUpdateSystem` (`sys_animation.rs`) — generic frame advance, ticks
   `current_frame = (current_frame + 1) % frame_count` every `FRAME_DURATION`
   (0.12s), **only when `running == true`**. The knight body has `running:
   false`, so this system does not drive the knight; `KnightControlSystem`
   drives it directly.
2. `KnightControlSystem` (`sys_knight_controls.rs`) — reads input and directly
   sets the knight body's `current_frame` (and position), and runs the attack
   state machine.
3. `OffsetUpdateSystem` (`sys_offsets.rs`) — reads the knight's current frame,
   then repositions/re-frames the held sword and shield to match.
4. `DrawSystem` (`sys_draw.rs`) — collects all `Animation` and `StaticImage`
   entities, sorts by `z_idx`, and draws them.

## The attack frames

The knight body has 6 frames (0-indexed in code). Mapping code index to the
human "frame N" (1-indexed):

| Code index | Human frame | Purpose | Source |
|-----------|-------------|---------|--------|
| 0 | frame 1 | idle | `IDLE_FRAME = 0` |
| 1 | frame 2 | **thrust** | `THRUST_FRAME = 1` |
| 2 | frame 3 | **swipe** | `SWIPE_FRAME = 2` |
| 3,4,5 | frames 4-6 | walk cycle | `WALK_FRAMES = [3,4,5]` |

This matches the user's description: **frame 2 is the thrust and frame 3 is the
swipe** (counting from 1; the code counts from 0, so they are indices 1 and 2).

`sys_knight_controls.rs` constants:

```rust
const IDLE_FRAME: usize = 0;
const THRUST_FRAME: usize = 1;
const SWIPE_FRAME: usize = 2;
const WALK_FRAMES: [usize; 3] = [3, 4, 5];
```

## How the sword/shield follow the attack frame (`sys_offsets.rs` + `entity_offsets.rs`)

The body sprite does not draw the full weapon; the sword and shield are separate
textures composited on top. `EntityOffsets::knight()` (`src/entity/entity_offsets.rs`)
declares, per body frame, where the sword/shield should sit and whether they are
visible:

- **Frame 0 (idle):** shield at (33,16), sword at (18,12), sword_frame 0; both visible.
- **Frame 1 (thrust):** sword jumps to (60,8) and uses sword_frame **1** (the
  extended/thrust sword pose); shield hidden, sword visible.
- **Frame 2 (swipe):** sword back at (18,12), sword_frame 0; **both shield and
  sword hidden** — the swipe pose in the body sprite already contains the weapon
  drawn in, so the separate sword/shield are turned off to avoid double-drawing.
- **Frames 3-5 (walk):** both visible, sword_frame 0.

`OffsetUpdateSystem` applies these each update by setting `image.position`,
`image.visible`, and `animation.current_frame` (for the sword) on the held
entities.

## Attack state machine (`sys_knight_controls.rs`)

Attacks are driven by a phase machine: `Idle -> Windup -> Strike -> Recovery -> Idle`.

- The attack frame is shown only during the **Strike** phase:
  `new_frame = current_attack.map(AttackKind::frame)` (line 208).
- Timings (seconds):
  - Thrust: windup 0.015, strike 0.10, recovery 0.15
  - Swipe:  windup 0.0075, strike 0.12, recovery 0.15
- Attack input is `Input::Attack`, bound to the **X key** in both clients
  (`clients/desktop/src/main.rs:16` and `clients/web/src/main.rs:16`).
- Consecutive attacks alternate: pressing attack while idle gives thrust, and
  the next gives swipe (`last_scheduled` logic, lines 169-175). Input can be
  buffered once during an active attack.

## Input handling (`src/input/mod.rs`)

Inputs are polled via a thread-local set filled each frame by the clients.
Relevant helpers: `input::down(Input)` (held), `input::down_once(Input)` (edge),
`input::up_once`, `input::down_once_every(input, cooldown_ms)`. The
`AssetPreviewScene::update` uses `Input::Jump` to toggle focus mode, and arrow
keys to move/cycle variants; `KnightControlSystem` uses arrow keys + `Attack`.

## Key takeaways for the plan

1. Frames are horizontal-strip sub-rects; a new attack frame just means setting
   the knight body's `Animation.current_frame` to 1 (thrust) or 2 (swipe).
2. The visible attack pose is a *composite* of three layers; the sword/shield
   offsets/visibility are driven per-frame by `EntityOffsets`, not baked into
   the body sprite (except the swipe, which hides the separate sword/shield).
3. The generic `AnimationUpdateSystem` only advances `running` animations; the
   knight's animation is advanced manually by `KnightControlSystem`.
4. `DrawTextureParams` already supports `rotation` and `pivot` if the swipe
   needs a real arc; neither is wired up yet.
5. Timing is done in `ctx.dt` (never `get_frame_time()` per AGENTS.md).

## References

- `docs/macroquad-graphics.md` — shape/texture/color drawing basics
- `docs/pixel-perfect-rendering.md` — render-target pipeline (2x overscan, Nearest)
- `docs/macroquad-shaders.md` — materials/uniforms (not needed for this task)
- `src/systems/sys_draw.rs` — actual `draw_texture_ex` calls
- `src/entity/animation.rs` — `Animation` component fields
- `src/entity/entity_offsets.rs` — per-frame sword/shield offsets + visibility
- `src/scene/asset_preview/sys_knight_controls.rs` — attack state machine + frame constants
- `src/scene/asset_preview/sys_offsets.rs` — applies frame offsets to held items
- `src/scene/asset_preview/sys_animation.rs` — generic frame advance
- macroquad `DrawTextureParams` docs: https://docs.rs/macroquad/latest/macroquad/texture/struct.DrawTextureParams.html
- macroquad `draw_texture_ex` docs: https://docs.rs/macroquad/latest/macroquad/texture/fn.draw_texture_ex.html
