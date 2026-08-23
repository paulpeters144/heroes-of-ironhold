# Research Topic
Find all constants in the game that are used for in-game entities (player, enemy), i.e. `const` declarations that drive entity behavior — like `REACH_DIST`.

## Entity constants (scene_one)

The in-game entities live in `src/scene/scene_one/`. All `const` declarations in that scene:

### Enemy system — `src/scene/scene_one/sys_enemy.rs`
- `IDLE_TIME: f32 = 1.2` — how long an enemy waits at a patrol endpoint before turning around (line 7, used at line 67).
- `REACH_DIST: f32 = 2.0` — distance threshold (px) at which an enemy is considered to have reached its patrol target (line 8, used at line 65).

### Hurt / damage blink system — `src/scene/scene_one/sys_hurt.rs`
- `BLINK_COUNT: u32 = 5` — number of blink toggles the player goes through after taking a hit (line 6, used at line 76).
- `BLINK_INTERVAL: f32 = 0.15` — seconds between each blink visibility toggle (line 7, used at line 90).

### Enemy factory (sprite/animation constants) — `src/scene/scene_one/factory.rs`
- `E3_TOTAL_FRAMES: usize = 14` — total frames in the E3 enemy sprite sheet (line 9).
- `IDLE_FRAMES: usize = 5` — frames in the enemy idle animation (line 10).
- `WALK_FRAMES: usize = 8` — frames in the enemy walk animation (line 11).
- `FALL_FRAMES: usize = 1` — frames in the enemy fall animation (line 12).
- `IDLE_FIRST: usize = 0` — starting frame index of the idle animation in the sheet (line 14).
- `WALK_FIRST: usize = 5` — starting frame index of the walk animation (line 15).
- `FALL_FIRST: usize = 13` — starting frame index of the fall animation (line 16).
- `IDLE_TIME: f32 = 0.15` — per-frame duration of the enemy idle animation (line 18).
- `WALK_TIME: f32 = 0.1` — per-frame duration of the enemy walk animation (line 19).

### Player sprite constants — `src/scene/scene_one/scene.rs`
- `IDLE_FRAMES: usize = 5` — player idle sprite frame count (line 22).
- `RUN_FRAMES: usize = 8` — player run sprite frame count (line 23).
- `JUMP_FRAMES: usize = 4` — player jump sprite frame count (line 24).
- `HURT_FRAMES: usize = 9` — player hurt sprite frame count (line 25).

## Duplicated / shadowed constant names (worth noting)

`IDLE_FRAMES` is declared in **three** places with the same value:
- `src/scene/scene_one/factory.rs:10` (enemy idle)
- `src/scene/scene_one/scene.rs:22` (player idle)
- (also `IDLE_TIME` in `factory.rs:18` vs `sys_enemy.rs:6` — two distinct meanings: enemy animation frame time vs enemy patrol idle wait time)

## Entity values that are hardcoded (NOT `const`)

These are entity-tunable numbers currently written inline as magic numbers — candidates for promotion to constants if the goal is to centralize entity tuning:

### Player `Physics` (set in `src/scene/scene_one/scene.rs:126-133`)
- `move_speed: 120.0`
- `gravity: 900.0`
- `jump_velocity: -320.0`
- `sprite_size: 32.0`
- `hitbox_w: 20.0`
- `hitbox_h: 30.0`

### Enemy `Physics` (set in `src/scene/scene_one/factory.rs:75-82`)
- `move_speed: 40.0`
- `gravity: 900.0`
- `jump_velocity: -320.0`
- `sprite_size: 32.0`
- `hitbox_w: 20.0`
- `hitbox_h: 30.0`

### Player sprite frame times (inline in `src/scene/scene_one/scene.rs:107-110`)
- idle `0.15`, run `0.075`, jump `0.1`, hurt `0.075` (passed as literals to `self.sprite(...)`).

### Enemy fall frame time (inline in `src/scene/scene_one/factory.rs:60`)
- `frame_time: 0.1`

### Other inline entity magic numbers
- `src/scene/scene_one/scene.rs:113` — `spawn_x = 48.0` (player spawn X).
- `src/scene/scene_one/scene.rs:114` — spawn Y computed from lowest collider `- 32.0`.
- `src/scene/scene_one/sys_animation.rs:40` — `idle_delay = 0.2` (delay before idle animation restarts).
- `src/scene/scene_one/sys_enemy.rs:68` — `p.a + 0.5` (patrol endpoint turn-around threshold).
- `src/scene/scene_one/sys_enemy.rs:106` — midpoint `(p.a + p.b) * 0.5` used to re-pick patrol target after hitting a wall.

## Non-entity constants (for completeness — not entity behavior)

These are `const` declarations elsewhere in the codebase, but they are UI/menu/asset related, not game-entity related:
- `src/scene/opening/sys_menu.rs`: `FLASH_DUR`, `EXPAND_DUR`, `HOLD_DUR`, `ITEM_STAGGER`, `ITEM_SLIDE_DUR`, `SELECT_DUR`, plus local `C1`/`C3` (easing constants).
- `src/scene/opening/sys_draw.rs`: `TITLE`, `PANEL_W`, `PANEL_H`, `ROW_H`, `ROW_GAP`, `HEADER_H`.
- `src/scene/opening/saves.rs`: `SAVE_SLOTS`.
- `src/scene/font_showcase/scene.rs`: `SAMPLE`.
- `src/ui/helpers.rs`: `CORNER_SIDES`.
- `src/access/ids.rs`: asset manifest arrays (`TEXTURES`, `FONTS`, `SOUNDS`, `FILES`, `SHADERS`, `IMAGES`).

## Summary

Game-entity `const` declarations are concentrated in `src/scene/scene_one/` across four files: `sys_enemy.rs` (2), `sys_hurt.rs` (2), `factory.rs` (9), and `scene.rs` (4) — 17 entity constants total. The player/enemy physics values, spawn positions, and frame times remain inline magic numbers rather than named constants.
