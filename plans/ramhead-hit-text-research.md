# Research Topic
How text is shown when the RamHead gets hit (floating damage numbers).

## Overview

The text that appears when a RamHead is hit is a **floating damage number**, plus a brief **white flash** overlay on the enemy sprite. Both are produced by a single system: `HandleAttackSystem` (`src/systems/sys_handle_attack.rs`). There is no separate "damage text" system; text and flash live together in this one file.

## Flow of events

1. **Detection** — `AttackHitSystem` (`src/scene/battle_test/sys_attack_hit.rs:24`) detects overlap between the Knight's sword `AttackArea` and any `RamHead`'s `Animation` rect. On a rising edge (`area.visible && !prev_visible`), it fires an `AttackEvent { attacker, targets }` on the shared `EventBus`. Targets are raw entity ids (`u64`), collected from `self.store.all::<RamHead>()`.

2. **Event queue** — `HandleAttackSystem::new` (`sys_handle_attack.rs:40`) subscribes to `AttackEvent` and only enqueues it into a `Rc<RefCell<VecDeque<AttackEvent>>>` (kept alive via `SubCollection`). The system is registered in the battle scene as **both** update and draw (`scene.rs:305-308`), and because it is `#[derive(Clone)]`, all mutable state (`flashing`, `damage_numbers`, `queue`) is behind `Rc<RefCell<...>>` so both clones share the same state.

3. **Processing** — `HandleAttackSystem::update` (`sys_handle_attack.rs:98`) drains the queue:
   - Damage = attacker `Knight` → `HeroStats.strength`, `max(1)`.
   - For each target id it inserts `target → FLASH_DURATION (0.15s)` into `flashing`.
   - It decrements `EnemyStats.hp` via an in-place `store.update` guard.
   - It computes a spawn position from the RamHead's `Animation` rect: `vec2(r.x + r.w*0.5, r.y - DAMAGE_TEXT_HEIGHT)` (`DAMAGE_TEXT_HEIGHT = 28.0`) — i.e. horizontally centered above the sprite.
   - It pushes a `FloatingNumber { pos, text: damage.to_string(), remaining: DAMAGE_TEXT_LIFETIME (1.0s) }` into `damage_numbers`.

4. **Lifetime ticking** — the same `update` advances `remaining -= ctx.dt` for both flashes and numbers, removing expired entries. Frame delta comes from `ctx.dt` (never `get_frame_time()`).

## Rendering (the actual text)

`HandleAttackSystem::draw` (`sys_handle_attack.rs:161`) does two things:

### Flash overlay
While `flashing` is non-empty, it binds `flash_material` (the `FlashWhiteFrag` shader, loaded in `load_flash_material`, `sys_handle_attack.rs:65`) and re-draws the RamHead's current animation frame with a custom blend. This is the white "hit" flash, not text.

### Damage number text
The `damage_numbers` are drawn with macroquad's `draw_text_ex`. For each number:
- `measure_text` computes dimensions; `x` is centered (`pos.x - dims.width*0.5`), `baseline = pos.y + dims.offset_y`.
- It draws the text **9 times**: 8 offset copies in black (the 8 neighbors `(-1,0),(1,0),(0,-1),(0,1),(-1,-1),(1,-1),(-1,1),(1,1)`) forming an outline, then once on top in the accent color.
- Font/color come from a `GameFont` built in `new()` from `TextStyle::new(FontTag::Body)` (Pixellari, nominal size 16) but overridden to `size: 24` and `color: Color::new(1.0, 0.2, 0.14, 1.0)` (a red-ish tone). Text is drawn at `font_size = self.font.size (24)`, `font_scale = 1.0`.

## Key facts / quirks

- **Coordinate space**: the number is positioned in *world space* using `Animation.position`/`rect()` (scene pass, camera-applied), not screen space. `draw` is added via `agg.add_draw`, so it renders during `draw_scene` under the camera transform. Note it does **not** use `crisp_text_params` / `snap_to_pixel` from `src/util/view_scale.rs` — those helpers exist for crisp UI text but are not applied here.
- **Font extraction happens in `new()`** (per `rules/conventions.md`, no asset access in `update`/`draw`). `GameFont` (font handle, size, color) is stored on the struct.
- The outline trick (8 black copies + 1 colored) is a manual text-stroke, since macroquad's `TextParams` has no built-in outline.
- Damage value is shown as a plain string (`damage.to_string()`); no crit/color variance is implemented.
- `FloatingNumber` currently stores a `String`; the draw path clones the whole vec (`iter().cloned().collect()`) to avoid holding the `RefCell` borrow during `draw_text_ex`.

## Relevant files

- `src/systems/sys_handle_attack.rs` — the system doing detection-side processing, flash, and damage text.
- `src/scene/battle_test/sys_attack_hit.rs` — fires `AttackEvent` when sword area overlaps a RamHead.
- `src/events.rs` — `AttackEvent` definition (`attacker: u64`, `targets: Vec<u64>`).
- `src/access/font.rs` — `FontTag`, `TextStyle`, `GameFont`, `Assets::get_font`.
- `src/util/view_scale.rs` — `crisp_text_params` / `snap_to_pixel` (crisp UI text helpers, *not* used by the damage numbers).
- `src/scene/battle_test/scene.rs:305-308` — system registration (update + draw).
- `src/systems/sys_health_bar.rs` — separate health-bar drawing (also RamHead-aware, uses `HealthBar` component, no text).
