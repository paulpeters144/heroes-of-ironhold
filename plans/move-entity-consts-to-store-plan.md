# Description
Move game-entity behavior constants out of global `const` declarations and into the entity store, so systems read entity values from entities/child entities instead of module-level globals. Global state is only used as a last-resort fallback when a value genuinely has no entity home.

# TODO
- [ ] Add runtime fields to `Patrol` (`reach_dist`, `idle_time`) and populate in `EnemyFactory::build_enemy_one`
- [ ] Add runtime fields to `Blink` (`blink_count`, `blink_interval`) and populate when the `Blink` child is created
- [ ] Update `sys_enemy.rs` to read `reach_dist`/`idle_time` from `Patrol` instead of `REACH_DIST`/`IDLE_TIME`
- [ ] Update `sys_hurt.rs` to read blink count/interval from `Blink` instead of `BLINK_COUNT`/`BLINK_INTERVAL`
- [ ] Create a sprite-definition entity (a `SpriteDef` component) in the store holding frame counts, first-frame indices, and frame times, and fetch it from the store where needed
- [ ] Delete all now-unused `const` declarations in `scene_one`
- [ ] Verify with `cargo build` / `cargo check`

# TODO Explanation

## Add runtime fields to `Patrol`
`REACH_DIST` and enemy `IDLE_TIME` are patrol behavior, so they belong on the `Patrol` component (`src/scene/scene_one/components.rs`). Add `reach_dist: f32` and `idle_time: f32` fields, set them per enemy in `EnemyFactory::build_enemy_one` (`src/scene/scene_one/factory.rs`) where the `Patrol` child is built, replacing the literals `2.0` and `1.2`. Each enemy's `Patrol` instance carries its own values.

## Add runtime fields to `Blink`
`BLINK_COUNT` and `BLINK_INTERVAL` are hurt/invulnerability behavior for the player, so they belong on the `Blink` component, per player so it stays tunable. Add `blink_count: u32` and `blink_interval: f32` fields and set them in `SceneOne::load` (`src/scene/scene_one/scene.rs`) where the `Blink` child is created, replacing literals `5` and `0.15`.

## Update `sys_enemy.rs`
Replace `REACH_DIST` and `IDLE_TIME` references with `patrol.reach_dist` and `patrol.idle_time` read from the `Patrol` clone already fetched in the update loop. No global const lookup.

## Update `sys_hurt.rs`
Replace `BLINK_COUNT`/`BLINK_INTERVAL` with values read from the player's `Blink` child. Since `sys_hurt` already fetches `Blink`, reset/initialize using the stored values instead of constants.

## Create sprite-definition entity in the store
The frame-count/index/time consts in `factory.rs` (`E3_TOTAL_FRAMES`, `IDLE_FRAMES`, `WALK_FRAMES`, `FALL_FRAMES`, `IDLE_FIRST`, `WALK_FIRST`, `FALL_FIRST`, `IDLE_TIME`, `WALK_TIME`) and `scene.rs` (`IDLE_FRAMES`, `RUN_FRAMES`, `JUMP_FRAMES`, `HURT_FRAMES`) are sprite-sheet slicing metadata. Rather than holding them as globals, create a new entity in the store carrying a `SpriteDef` component (`first_frame`, `frame_count`, `frame_time`, plus the image `Id` needed to resolve the texture). Add these entities (e.g. one per sprite animation for the player and enemy, or a small set of `SpriteDef` entities) in `SceneOne::load` / `EnemyFactory`. Anywhere the sprite slicing info is needed, look it up from the store instead of referencing a global const.

## Delete unused consts
After all references are migrated, remove the `const` declarations at the top of `sys_enemy.rs`, `sys_hurt.rs`, `factory.rs`, and `scene.rs`.

## Verify
Run `cargo build` (or `cargo check`) from the repo root to confirm nothing else referenced the removed consts and the store-based reads compile.

# Open Questions

# Out of Scope
- We are not converting non-entity constants (menu/UI constants in `scene/opening`, `ui/helpers.rs`, asset manifests in `access/ids.rs`) — only game-entity constants.
- We are not changing the `SystemAgg`/system scheduling architecture.
- We are not introducing a new serialization/asset format; only moving values into the existing entity store.
- Physics values (`move_speed`, `gravity`, `jump_velocity`, `sprite_size`, `hitbox_w`, `hitbox_h`) are not part of this refactor — they already come from the `Physics` component in the store and are not global constants.
