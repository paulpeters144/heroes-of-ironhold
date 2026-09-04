# Description
Add a charge-based dash: the knight's `Dash` component gains a finite dash count (charges) plus recovery state. Consuming a dash immediately begins a 15-second recovery timer per charge. Add a dash UI symbol on the left side of the screen, below the top-left HUD, showing remaining charges and a WoW-style cooldown counter (radial sweep + seconds countdown) while charges are recovering.

# TODO
- [ ] Extend the `Dash` component to hold charge data and recovery timers
- [ ] Rework `PlayerDashSystem` to consume charges instead of removing/re-adding the component
- [ ] Implement per-charge recovery (15s) that starts immediately when a dash is consumed
- [ ] Add a `DashUiDrawSystem` that draws the dash symbol + charge count + cooldown counter below the HUD
- [ ] Wire the new system into `BattleTestScene` (update + draw_ui) and preload any new assets

# TODO Explanation

## Extend the `Dash` component to hold charge data and recovery timers
Currently `Dash { dir: Vec2, time: f32 }` (`src/entity/knight/components.rs:24-28`) models only the *active* dash state, and `PlayerDashSystem` spawns a `Dash` child on the `Knight` entity when a dash starts, then removes it when `time` runs out. We need `Dash` to also hold:
- `charges: u32` (current available charges) and `max_charges: u32`
- recovery state, e.g. `recovering: Vec<f32>` (one timer per missing charge) or a `charge_timers` queue, with a `RECOVER_SECS = 15.0` constant added to `src/entity/knight/movement.rs`.

Decision needed: extend the existing `Dash` component vs. a separate `DashCharges` component (see Open Questions). Since the user asked for "a component called Dash that holds the data for the dash", the working assumption is to extend `Dash` and have it always attached to the `Knight` (not spawned/removed per dash). `dir`/`time` continue to describe the in-flight dash (e.g. `time > 0` means dashing).

## Rework `PlayerDashSystem` to consume charges instead of removing/re-adding the component
`src/scene/asset_preview/sys_player_dash.rs` currently: detects double-tap (only when `!attacking && !has_dash`), spawns a `Dash` child, moves `Animation.position += dir * DASH_SPEED * dt`, forces `IDLE_FRAME`, decrements `time`, and removes the `Dash` child when `time <= 0`. New behavior:
- `Dash` exists on the knight from spawn with `charges = max_charges`.
- On double-tap (when `!attacking` and not currently dashing i.e. `time <= 0`), if `charges > 0`: set `dir`, set `time = DASH_DURATION`, `charges -= 1`, and **immediately** start recovery for that charge (push a 15s timer).
- Active-dash movement stays as-is (move by `DASH_SPEED * dt`, idle frame).
- When `time` expires, stop dashing but keep the `Dash` component (no removal).

## Implement per-charge recovery (15s) that starts immediately when a dash is consumed
Recovery decrements each pending timer by `ctx.dt`; when a timer hits 0, `charges += 1` (capped at `max_charges`) and that timer is removed. This is a small update step, either inside `PlayerDashSystem` or a separate `DashRecoverySystem`. The user said "when the player does a dash, then it should immediately start recovering" — recovery begins at the moment the charge is spent, not when the dash animation ends. Model choice (independent parallel timers vs. one-at-a-time) is an Open Question; WoW charge systems recover each charge independently in parallel.

## Add a `DashUiDrawSystem` that draws the dash symbol + charge count + cooldown counter below the HUD
New draw system (e.g. `src/scene/battle_test/sys_dash_ui.rs`) following the pattern of `sys_hud.rs`/`sys_skills_bar.rs` (raw macroquad + `crate::ui::draw_rounded_rect`), drawn from `BattleTestScene::draw_ui`:
- Anchored on the **left** edge, **below** the top-left HUD block (portrait at y=8, height 50, XP line ends around y=58 — so place the symbol below that, e.g. y ≈ 65+).
- A square symbol/well with the dash icon (procedural or texture — see Open Questions).
- Charge count: show `charges` (e.g. as a number, or N small pips) so the player sees how many dashes remain.
- Cooldown counter (WoW-style): while at least one charge is recovering, overlay a **radial sweep** (a pie/arc that fills as recovery progresses — clockwise like WoW) tinted/darkened over the icon, with the remaining seconds in the center (e.g. "14"), snapped-to-pixel crisp text like the HUD. When full, icon is bright with no counter.
- If charges can recover in parallel, the display shows the *soonest* completing charge's remaining time (standard charge-display behavior).

## Wire the new system into `BattleTestScene` (update + draw_ui) and preload any new assets
- In `BattleTestScene::new` add the recovery/update step; in `load` construct the `DashUiDrawSystem` and store it like `hud`/`skills_bar`; call it from `draw_ui`.
- If a texture is used for the symbol, register it in `src/access/ids.rs` (image category + manifest), add it to `BattleTestScene::load`'s `preload` list, and follow the AGENTS.md asset rules.
- Ensure the knight is spawned with an initialized `Dash` (charges) component.

# Open Questions
- How many dash charges should the player have (value of N)? Working assumption: 3.
- Should each missing charge recover on its own **parallel** 15s timer (WoW-style, multiple can recover at once), or one-at-a-time? Working assumption: parallel independent timers.
- Extend the existing `Dash` component, or keep `Dash` for the active dash and add a separate charge/recovery component? Working assumption: extend `Dash`.
- Dash symbol art: draw it procedurally (like the skills-bar icons) or use an image asset? Working assumption: procedural, no new asset.
- Should the cooldown counter show only while recovering, or always show the next-charge progress even when some charges remain? Working assumption: always show the soonest recovering charge's progress when `charges < max`.
- Keep dash mechanics otherwise identical (double-tap, cardinal directions, blocked during attack)?

# Out of Scope
- No dash collision, i-frames, or dash-cancelling mechanics.
- No changes to how the HUD portrait/HP/MP/XP or skills bar are drawn.
- No key/input binding changes.
