# Research Topic
How the UI currently works and how the player dash works in `heroes-of-ironhold`.

---

## Overview of the project

- Workspace root is also the `heroes-of-ironhold-core` library crate (`Cargo.toml` at repo root).
- Platform entry points live in `clients/desktop/` and `clients/web/`; both map macroquad keys into a shared input abstraction (`crate::input`) and then call `core::update`/`core::draw` (see `clients/desktop/src/main.rs:9-23`).
- Rendering/state is driven by a `Manager` (`src/manager/mod.rs`) which owns the current `Scene`, a `GameCamera`, and two full-screen shader materials (`pixel_snap`, `jitter_free`).
- Virtual resolution is 640x360 (`v_width`/`v_height`); window is 1280x720 with a 2x overscan render target (`src/util/config.rs:13-25`).

---

## UI: architecture

### The `src/ui/` module (immediate-mode command buffer)

Files: `mod.rs`, `core.rs`, `rect.rs`, `params.rs`, `helpers.rs`, `style.rs`, `text.rs`, `cmd.rs`.

The `UI` struct (`src/ui/core.rs`) is an **immediate-mode, retained command-buffer** UI:
1. `UI::new(cfg, style)` stores the virtual view size and style.
2. `UI::begin(ctx)` clears the command list and computes the mouse position in virtual space (scaled by `view_scale`).
3. Widget calls (`rect()`, `label()`, `image()`) push a `Cmd` into `self.cmds` and return a `Rect` describing the area they will occupy.
4. `UI::end()` iterates `cmds` and actually draws.

`Cmd` enum (`src/ui/cmd.rs`) has four variants:
- `Solid { x, y, w, h, color, radius }`
- `Outlined { x, y, w, h, fill, border, thickness, radius }`
- `Text { x, y, text, font }`
- `Image { x, y, w, h, texture }`

`RectBuilder` (`src/ui/rect.rs`) is the widget entry point (`ui.rect()`), with methods:
- `solid(Solid)`, `outlined(Outlined)`
- `centered_solid(CenteredSolid)`, `centered_outlined(CenteredOutlined)` — center on the virtual view
- `default_outlined(DefaultOutlined)`, `default_centered_outlined(DefaultCenteredOutlined)` — pull fill/border/radius from `Style`

`params.rs` defines the parameter structs (`Label`, `Solid`, `Outlined`, `CenteredSolid`, `CenteredOutlined`, `DefaultOutlined`, `DefaultCenteredOutlined`, `Image`). Note every params struct carries an `ctx: &Context` field that is currently **unused** in the implementations.

`Style` (`src/ui/style.rs`) is `{ fill, border, border_width, radius }` with `Default` = white fill / black border / 1px / radius 0, plus a `rounded(radius)` builder.

### Primitives (`src/ui/helpers.rs`)

- `draw_rounded_rect(x, y, w, h, radius, color)`: builds rounded rects from 2 rectangles + 4 corner circles; `radius <= 0` falls back to `draw_rectangle`. Radius is clamped to half the min dimension.
- `draw_rounded_rect_lines(...)`: 4 straight edge lines + 4 corner arcs (`draw_arc`, `CORNER_SIDES = 16`), or `draw_rectangle_lines` when radius is 0.

### Text (`src/ui/text.rs` + `label`)

- `wrap_text(font, text, max_width, scale)` word-wraps at `max_width`; `max_width <= 0` disables wrapping.
- `UI::label` measures wrapped text line-by-line (`measure_text`), computes height as `lines * font.size`, and pushes a single `Cmd::Text`; it returns a `Rect` (width = max measured line width).
- `UI::end` renders `Text` per line, snapping positions with `view_scale::snap_to_pixel`.

### Virtual-screen scaling (`src/util/view_scale.rs`)

- `view_scale(v_width, v_height) -> (scale, offset)`: scale = min(screen/v), offset centers the view (letterboxing).
- `crisp_text_params(nominal_size, scale) -> (font_size, font_scale)`: rasterizes glyphs at on-screen size and draws back down by `1/scale` so atlas texels map 1:1 to screen pixels.
- `snap_to_pixel(v, scale)`: rounds `v * scale` to whole pixels and divides back, for crisp nearest-filtered text.

### Render pipeline (`src/manager/mod.rs`)

`Manager::draw` (`src/manager/mod.rs:161-168`) runs four passes:
1. `begin_scene_pass` — sets the game camera (`camera_at(ctx.cam_target)`) and clears black.
2. `draw_scene` — applies optional `pixel_snap` material, calls `scene.draw(ctx)`.
3. `begin_screen_pass` — `set_default_camera()`, clears black.
4. `blit_target` — blits the overscan render target to the window with the `jitter_free` material, then `draw_ui`.

`draw_ui` (`src/manager/mod.rs:198-222`) builds a `Camera2D` mapping virtual 640x360 space onto the letterboxed viewport (`zoom = 2/v_width, 2/v_height`, viewport offset from `view_scale`) and calls `scene.draw_ui(ctx)`. So `Scene::draw_ui` receives a context where drawing coordinates are in virtual 640x360 units; scaling happens via the camera.

### Scene trait

`Scene` (`src/scene/traits.rs`) has `load`, `update`, `draw`, and default-noop `draw_ui` / `dispose`. Only scenes that draw HUD override `draw_ui`.

### Where UI code actually lives (three distinct places)

1. **`src/ui/` module** — the generic immediate-mode UI. Currently **only used by `AssetPreviewScene`** (`src/scene/asset_preview/scene.rs:362-450`): it creates a `UI`, calls `begin`, draws three selection boxes via `ui.rect().outlined(Outlined { ... })`, and calls `ui.end()`. The boxes border the outfit/shield/sword preview textures. `box_style()` returns `(fill, border, thickness)` depending on `focus_mode`/`focus`.
2. **`HudDrawSystem`** (`src/scene/battle_test/sys_hud.rs`) — **does not use the `src/ui` module**; it draws directly with macroquad and local helper `pill()`/`pill_fill()`. Layout constants are hardcoded in virtual space. Shows a portrait plate, HP/MP pill bars (gradient fill + gloss), and an XP line with level medallion.
3. **`SkillsBarDrawSystem`** (`src/scene/battle_test/sys_skills_bar.rs`) — also **does not use the `src/ui` module**; it uses `crate::ui::draw_rounded_rect` plus raw macroquad. Draws a framed bar of square skill slots anchored bottom-center, with procedural vector icons and brass key-cap badges.

There is **no shared widget/button abstraction** across HUD/skills bar; each is bespoke drawing code.

---

## UI: the HUD (`src/scene/battle_test/sys_hud.rs`)

- Constructed in `BattleTestScene::load` with `HudDrawSystem::new(cfg, assets, store)`; called from `BattleTestScene::draw_ui`.
- Reads `HeroStats` (child of the `Knight` entity, `src/entity/knight/components.rs:30-55`, defaults: hp 145/150, mp 80/100, xp 2500/5000, level 5) via `store.first::<Knight>()` + `get_child::<HeroStats>`.
- Layout (virtual 640x360, top-left anchored):
  - `MARGIN = 8`, `PORTRAIT_SIZE = 50`, `FACE_SIZE = 40`, `PORTRAIT_RADIUS = 5`
  - Bars right of portrait: `BAR_X = 8 + 50 + 6 = 64`, `BAR_W = 126`, `BAR_H = 13`, `BAR_GAP = 4`; HP at `y=8`, MP below; XP line height 5 with a `MEDAL_R = 9` medallion holding the level digit.
- Palette constants define a "dark steel frame / warm steel rim / deep navy track" look; HP/MP/XP use `(light, dark)` gradient pairs with a translucent white gloss band.
- Key helpers:
  - `pill(x,y,w,h,radius,color)` — same rounded-rect construction as `ui::helpers::draw_rounded_rect` but local (duplicated logic).
  - `pill_fill(...)` — gradient fill split into 4 horizontal bands, rounded end caps, plus gloss.
  - `text(...)` — 8-direction ink shadow + near-white label; `raw_text` snap-to-pixel + `crisp_text_params`.
  - `portrait()`, `bar()`, `xp_line()` compose the frame → rim → track layering.
- Font: `assets.get_font(&TextStyle::new(FontTag::Body))`, size 12, near-white color (`src/scene/battle_test/sys_hud.rs:137-142`). Nearest filtering is set on the face texture.
- Bar labels ("HP"/"MP") are left-aligned inside the bar; numeric `value` (`hp/max_hp` formatted) is right-aligned.

## UI: the skills bar (`src/scene/battle_test/sys_skills_bar.rs`)

- Constructed in `BattleTestScene::load`; called from `draw_ui`.
- Entity model (`src/entity/skills/components.rs`):
  - `SkillsWidget` — parent marker.
  - `Skill { selected: bool, key: Option<char> }` — one slot, child of widget.
  - `SkillIcon { kind: SkillIconKind }` — child of `Skill`; `SkillIconKind` = Sword/Shield/Potion/Fireball/Crossed.
- `SkillsFactory::spawn(store, slots)` (`src/entity/factory_skills.rs`) builds `SkillsWidget -> Skill -> SkillIcon`; slots with `icon: None` are empty and get no key; icons get sequential keys `'a'`, `'b'`, ... (`next_key` counter). The battle-test scene spawns: Sword, Shield, Potion, Crossed, empty.
- `slots()` snapshots `Skill`/`SkillIcon` out of the store into `SlotView { icon, selected, key }`.
- Layout: `SLOT = 28`, `GAP = 3.2`, `PAD = 4`, `BOTTOM = 5`, `BAR_RADIUS = 3.2`, `SLOT_RADIUS = 2`; bar width = `PAD*2 + n*SLOT + (n-1)*GAP`, centered on `v_width`, anchored `v_height - BOTTOM - h`.
- Each slot: dark frame rect, rounded well with edge, top highlight + bottom shade lines, optional orange select tint (`SELECT`, `SELECT_GLOW`, `SELECT_TINT`), then icon (or dim `+` when empty), select glow rectangles, then key-cap badge.
- Icons are **procedurally drawn with macroquad primitives** (lines/triangles/circles): sword, kite shield, potion, fireball, crossed strokes. Key cap is a rounded rect "physical key" bleeding below the slot (`KEY_BLEED = 9`).

---

## Player dash

### Data & constants

- `Dash` component (`src/entity/knight/components.rs:24-28`): `{ dir: Vec2, time: f32 }`.
- Constants (`src/entity/knight/movement.rs`):
  - `MOVE_SPEED = 125.0`, `MOVE_SPEED_VERTICAL = 93.75`, `REVERSE_MULT = 0.75`
  - `WALK_FRAME_DURATION = 0.14`, `DOUBLE_TAP_WINDOW = 0.25`
  - `DASH_SPEED = 450.0`, `DASH_DURATION = 0.15`

### Input abstraction (`src/input/mod.rs`)

- Thread-local `InputState { current, previous, now_ms, last_fired }`.
- `Input` enum: Enter, Up, Down, Left, Right, Jump, Attack, Shift.
- `set(input, down)` fills `current` from the client each frame; `end_frame(dt)` moves `current` → `previous`, clears `current`.
- `down`, `up`, `down_once` (edge), `up_once`, `down_once_every(input, cooldown_ms)`.
- Key bindings (`clients/desktop/src/main.rs`): arrows = direction, Space = Jump, **X = Attack**, Left/Right Shift = Shift.

### Dash logic (`src/scene/asset_preview/sys_player_dash.rs`)

`PlayerDashSystem` implements `Update` and runs **before** `KnightControlSystem` in the aggregator (see ordering below). State: `last_tap: Option<Input>`, `tap_timer: f32`.

Each update:
1. Decrement `tap_timer` by `ctx.dt` (clamped at 0).
2. Find the `Knight`; check `has_dash = get_child::<Dash>().is_some()` and `attacking = animation.current_frame ∈ {THRUST_FRAME, SWIPE_FRAME}`.
3. **Double-tap detection** (only if `!attacking && !has_dash`): iterate `(Left,(-1,0))`, `(Right,(1,0))`, `(Up,(0,-1))`, `(Down,(0,1))`; on `input::down_once(input)`:
   - if `last_tap == Some(input)` and `tap_timer > 0` → spawn a `Dash { dir, time: DASH_DURATION }` child on the `Knight` entity; clear `last_tap`/`tap_timer`.
   - else record `last_tap = input`, `tap_timer = DOUBLE_TAP_WINDOW`.
   - `break` after the first pressed direction.
4. **Dash application**: read `dash.dir`, `remaining = dash.time - ctx.dt`, and refs; then `store.update::<Animation>` to move `position += dir * DASH_SPEED * dt` and force `current_frame = IDLE_FRAME`.
5. If `remaining <= 0` → `store.remove(&[dash_ref])`; else write `dash.time = remaining`.

Notes:
- Dash direction is axis-aligned (cardinal only, one key at a time).
- Dash movement is applied directly to `Animation.position` (no collision/obstacle check).
- Dash is blocked while an attack frame (thrust/swipe) is active, and double-tap is disabled while already dashing.

### Movement & interaction with dash (`src/scene/asset_preview/sys_knight_controls.rs`)

`KnightControlSystem` handles facing, walk animation, and the attack combo state machine, and is where dash presence affects movement:
- Reads arrow keys; horizontal `dir` (-1/0/+1) and vertical `dir_y`; Shift alone (no dir) does nothing for facing.
- Facing (`Facing` component) updates only when moving horizontally and Shift not held; a facing change cancels the buffered combo (`combo_reset`).
- Attack combo: `X` (`Input::Attack`) buffers Thrust → Swipe alternating via `last_scheduled()`; `AttackPhase` state machine (Windup 0.05 → Strike 0.15 → Recovery 0.05) drives `current_frame` to `THRUST_FRAME`/`SWIPE_FRAME` during Strike.
- Movement frame selection:
  - Strike → attack frame; Recovery → idle.
  - Otherwise, **if `Dash` exists** → walk timer reset, `IDLE_FRAME` (dash overrides walking visuals, but `PlayerDashSystem` already moved the position).
  - Else if moving → walk frames (`WALK_FRAMES` cycle; reverse order when Shift + facing-opposite direction = backward walking at `REVERSE_MULT` speed).
  - Else → idle.
- Writes `animation.position` (rounded) and `animation.current_frame`.

### Dash visual effect (`src/scene/asset_preview/sys_dash_fx.rs`)

`DashFxDrawSystem` implements `Draw`; draws afterimages only while a `Dash` component exists:
- Loads a material from `DashFxVert` + `DashAfterimageFrag` with a `tint` float4 uniform and custom alpha blending (source-alpha color blend; keep-alpha blend for alpha channel). Failure is tolerated (logs and disables).
- `outfit_dominant_color(texture)`: samples the central quarter of the outfit texture, builds a 3-bit-per-channel histogram, skips transparent and near-black (outline) pixels, picks the most common color, normalizes to max channel = 1.0. This tint is computed once in `load` and stored in an `Rc<Cell<Color>>` shared with the scene.
- `draw`: `progress = (1.0 - dash.time / DASH_DURATION).clamp(0,1)`; for each of `GHOST_COUNT = 3` ghosts, offset `-dir * GHOST_SPACING * (i+1)` (13.0 spacing), alpha = `GHOST_BASE_ALPHA (0.7) * (1-t)^2 * progress`, tint set per ghost, draws the knight's current frame texture behind it.

### Supporting systems (battle-test scene)

From `BattleTestScene::new` (`src/scene/battle_test/scene.rs:45-53`), update order is:
1. `PlayerDashSystem` (dash spawning + movement)
2. `KnightControlSystem` (facing/walk/attack; respects `Dash` presence for frame)
3. `KnightFacingLockSystem` (`src/scene/battle_test/sys_facing_lock.rs`) — locks facing for `FACE_LOCK_SECS = 0.25` when an attack frame is entered, preventing turn-around mid-attack.
4. `EnemyAiSystem` (`sys_enemy_ai.rs`)
5. `AnimationUpdateSystem` (`sys_animation.rs`)
6. `OffsetUpdateSystem` (`sys_offsets.rs`) — positions shield/sword children from frame offsets (`knight_offsets()` in `src/entity/knight/frames.rs`).
7. `AttackEffectSystem` (`sys_attack_effects.rs`)
8. `CameraOrbSystem` (`sys_orb.rs`) — places a camera-follow orb 25px in front of the knight's facing; `CameraSystem` (`sys_camera.rs`) springs toward it (critically damped, stop zone, clamped to map).

Draw order: `MapDrawSystem` → `DrawSystem` → `AttackEffectDrawSystem` → `DashFxDrawSystem`; HUD and skills bar are drawn separately in `draw_ui`.

### Dash summary of numbers

| Constant | Value | Meaning |
|---|---|---|
| `DOUBLE_TAP_WINDOW` | 0.25 s | max gap between the two taps |
| `DASH_SPEED` | 450 px/s | dash velocity (virtual units) |
| `DASH_DURATION` | 0.15 s | dash length (~67.5 virtual px) |
| `GHOST_COUNT` / `GHOST_SPACING` | 3 / 13 px | afterimage count & spacing |
| `GHOST_BASE_ALPHA` | 0.7 | max afterimage alpha |

---

## Key observations / gaps

- The generic `src/ui` module is effectively **under-used**: only `AssetPreviewScene` uses it, and only for outlined boxes. The real in-game HUD (`sys_hud.rs`) and skills bar (`sys_skills_bar.rs`) bypass it and draw raw macroquad, sharing only `ui::draw_rounded_rect`. There is duplicated rounded-rect logic (`helpers::draw_rounded_rect` vs HUD `pill`).
- The `ctx: &Context` field on every `src/ui/params.rs` struct is carried but unused; `UI::begin(ctx)` also ignores its `_ctx` argument.
- No button/interaction/hit-testing exists in the UI — widgets only draw and return `Rect`s; the skills bar has no input handling yet (keys are assigned but unused; `Skill.selected` is static).
- Dash is a self-contained ECS flow: `PlayerDashSystem` writes `Dash`, moves the `Animation`, and the `DashFxDrawSystem` reads `Dash` for afterimages. No cooldown, i-frames, or collision logic.
- Dash and movement both write `Animation.position` in the same frame (dash first, then controls); controls round the position and do not add walk movement while `Dash` is present, so they don't fight.
