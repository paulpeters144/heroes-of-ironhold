# Description
Add a charge-based dash implemented as **one self-contained `DashSystem`** that owns all dash concerns: update logic (charge consumption, active-dash movement, recovery), scene-space afterimage drawing, and screen-space dash UI drawing. The knight's `Dash` component gains a finite dash count of **3 charges** plus recovery state. Consuming a dash immediately begins a 15-second recovery that refills one charge at a time, Diablo 3 style — a single serial countdown that keeps refilling until charges are back at max. A dash UI symbol is drawn on the left side of the screen, below the top-left HUD, showing remaining charges and a Diablo 3-style radial sweep that circles the widget while charges recover (no numeric countdown text).

# TODO
- [ ] Create a single modular `DashSystem` that consolidates dash update + afterimage draw + dash UI draw
- [ ] Extend the `Dash` component to hold charge data and recovery state (all dash state lives in ECS)
- [ ] Implement charge consumption + a single serial 15s recovery countdown (Diablo 3 style), starting immediately when a dash is consumed
- [ ] Draw the dash symbol + charge count + Diablo 3-style circular sweep below the top-left HUD (procedural icon)
- [ ] Wire `DashSystem` into `BattleTestScene` (update + draw + draw_ui)

# TODO Explanation

## Create a single modular `DashSystem` that consolidates dash update + afterimage draw + dash UI draw
Today dash is split across three places: `PlayerDashSystem` (`src/scene/asset_preview/sys_player_dash.rs`, `Update` only), `DashFxDrawSystem` (`src/scene/asset_preview/sys_dash_fx.rs`, `Draw` only, afterimages), and there is no dash UI. Consolidate **everything** dash-related into one `DashSystem` module at `src/scene/battle_test/sys_dash.rs`: double-tap detection, charge consumption, active-dash movement, recovery, afterimage drawing (including the afterimage material, `outfit_dominant_color`, and the dash tint color), and the dash UI. Keep as much as possible colocated in this one system. `AssetPreviewScene` keeps its existing `PlayerDashSystem`/`DashFxDrawSystem` unchanged for now.

Key architecture facts this must respect (verified in the codebase):

1. **`SystemAgg` keeps update and draw systems in separate vecs** (`src/systems/system_agg.rs`): `updates: Vec<Box<dyn Update>>` and `draws: Vec<Box<dyn Draw>>`, registered via `add_update`/`add_draw`, each taking ownership. A single *type* can implement both traits (e.g. `CameraOrbSystem` in `src/scene/battle_test/sys_orb.rs` implements both `Update` and `Draw`), but it is added twice, as two separate instances. A single *instance* cannot serve both roles through `SystemAgg`.
2. **UI must render in the `draw_ui` pass, not the `Draw` pass.** `Draw`-trait systems run inside `Manager::draw_scene` under the **game camera** (`src/manager/mod.rs:161-191`); the dash charge symbol must be fixed on screen, so it has to go through `Scene::draw_ui`, which `Manager::draw_ui` (`src/manager/mod.rs:198-222`) invokes under a virtual 640x360 camera. This is exactly how `HudDrawSystem` (`sys_hud.rs`) and `SkillsBarDrawSystem` (`sys_skills_bar.rs`) work: they are scene-stored structs with a `draw(&self, ctx)` method called from `BattleTestScene::draw_ui`.
3. **All mutable dash state lives in the ECS, not on the system** — this is what makes the split-registration pattern safe. Systems only hold transient derived state over a shared `Rc<EStore>` (e.g. `AnimationUpdateSystem.elapsed`, `AttackEffectSystem.prev_frame`).

Consequence/design: `DashSystem` is a **stateless-over-ECS** struct holding only `Rc<EStore>` plus immutable draw resources (`Rc<Config>`, afterimage `Material`, font/icon), and it:
- implements `Update` (double-tap detection, charge consumption, active-dash movement, recovery) — registered via `agg.add_update(DashSystem::new(...))`;
- implements `Draw` (afterimages, scene space) — registered via `agg.add_draw(DashSystem::new(...))`;
- exposes `draw_ui(&self, ctx: &Context)` (symbol + charges + circular sweep) — stored on the scene as `dash_ui: Option<DashSystem>` and called from `BattleTestScene::draw_ui`.

Multiple instances of the same type stay coherent because the real state is the `Dash` component in the store. The only non-ECS state (double-tap `last_tap`/`tap_timer`) lives on the update instance, which is the sole writer.

## Extend the `Dash` component to hold charge data and recovery state (all dash state lives in ECS)
Currently `Dash { dir: Vec2, time: f32 }` (`src/entity/knight/components.rs:24-28`) models only the in-flight dash, and `PlayerDashSystem` spawns a `Dash` child on the `Knight` when a dash starts and removes it when `time` expires. Change `Dash` to be **permanently attached to the knight** and hold:
- `charges: u32` and `max_charges: u32` — the player starts with **3 charges** (`max_charges = 3`, `charges = 3`).
- active-dash state: `dir: Vec2`, `time: f32` (`time > 0` means dashing)
- recovery state: `recovery: f32` — a single countdown toward the next charge (0 = no pending recovery), with a `RECOVER_SECS = 15.0` constant added to `src/entity/knight/movement.rs` alongside `DASH_SPEED`/`DASH_DURATION`.

Because the knight currently has no `Dash` until the first double-tap, the spawn path must be updated to attach an initialized `Dash { charges: 3, max_charges: 3, dir: ZERO, time: 0, recovery: 0.0 }` when the knight is created (or the system lazily creates it if absent).

## Implement charge consumption + a single serial 15s recovery countdown (Diablo 3 style)
In `DashSystem::update` (replacing the body of `PlayerDashSystem`):
- Decrement `tap_timer` by `ctx.dt`.
- Double-tap detection (mechanics unchanged): when `!attacking` and not currently dashing (`time <= 0`), on a double-tap of a cardinal direction (`Left`/`Right`/`Up`/`Down`) with `charges > 0`: set `dir`, `time = DASH_DURATION`, `charges -= 1`, and **start/restart the recovery countdown** (`recovery = RECOVER_SECS`) — recovery begins the moment the charge is spent, not when the dash animation ends.
- Active-dash movement: `Animation.position += dir * DASH_SPEED * dt`, force `IDLE_FRAME`; decrement `time`; when `time` hits 0 the dash ends (component stays).
- Recovery (serial, Diablo 3 style): while `charges < max_charges`, decrement `recovery` by `ctx.dt`; when it reaches 0, `charges += 1`; if `charges < max_charges`, reset `recovery = RECOVER_SECS` and keep counting down. Once `charges == max_charges`, clear `recovery` to 0 and stop. There is exactly **one** countdown at a time — charges refill one at a time, and the counter never stops until full.

The knight controls already force idle while a `Dash` exists (`sys_knight_controls.rs`), and `KnightFacingLockSystem`/attack gating remain unchanged.

## Draw the dash symbol + charge count + Diablo 3-style circular sweep below the top-left HUD
In `DashSystem::draw_ui`, following the raw-macroquad + `crate::ui::draw_rounded_rect` style of `sys_hud.rs`/`sys_skills_bar.rs`:
- Anchor on the **left edge, below the top-left HUD block** (portrait occupies y=8..58; the XP line ends around y≈58, so place the symbol at y ≈ 65+, x = MARGIN, matching HUD constants).
- Draw a square well/frame (matching the HUD steel-frame palette: `FRAME`/`RIM`/`TRACK` from `sys_hud.rs` or a dash-specific tint) with a **procedurally drawn dash icon** inside (no image asset — draw the glyph with macroquad primitives, in the same spirit as the `SkillsBarDrawSystem` icons).
- **Charge count**: show `charges` (number and/or pips) so the player sees remaining dashes.
- **Cooldown animation (Diablo 3 style, no numeric text)**: while `charges < max_charges`, overlay a darkened radial wedge over the icon whose edge **sweeps clockwise around the widget** as recovery progresses. Implementation: a filled pie/arc (or a clipped circle drawn with several triangle/arc segments, like the `CORNER_SIDES` approach in `ui/helpers.rs`) whose remaining fraction = `recovery / RECOVER_SECS`; as `recovery` counts down to 0, the dark overlay shrinks and the sweep circles around until the charge is restored. The direction/start angle can be tuned to match Diablo 3 (sweep starts at 12 o'clock and moves clockwise).
- When all charges are full, draw the icon bright with no overlay. Because recovery is a single serial countdown that never stops until full, the circling sweep is always shown whenever `charges < max_charges` — it is the visual countdown to the next charge. No seconds text is rendered.

## Wire `DashSystem` into `BattleTestScene` (update + draw + draw_ui)
- In `BattleTestScene::load` (mirroring how `CameraSystem` is added there): construct `DashSystem` with `store`, `cfg`, and `assets`; `self.agg.add_update(...)` and `self.agg.add_draw(...)`; store `self.dash_ui = Some(...)` for `draw_ui`.
- Remove `PlayerDashSystem` and `DashFxDrawSystem` from `BattleTestScene::new`/`load` (and their imports); keep the `dash_color` cell if afterimages still need it (or move dominant-color computation into `DashSystem`).
- `BattleTestScene::draw_ui` calls `dash_ui.draw_ui(ctx)` after the HUD.
- No new assets: the dash icon is procedural, so nothing is added to `src/access/ids.rs` or the preload list.
- Ensure the knight spawns with an initialized `Dash` component.

# Out of Scope
- No dash collision, i-frames, or dash-cancelling mechanics.
- No changes to how the HUD portrait/HP/MP/XP or skills bar are drawn.
- No key/input binding changes.
- No changes to `SystemAgg` or the `EStore`/`pico_entity_store` APIs.
