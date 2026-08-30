# Description
Add a facing concept to the knight so it can look left or right, driven by horizontal movement input. When the knight faces left, its body, shield, and sword must mirror correctly (via `flip_x` plus mirrored child offsets), attack effects must follow, and the camera orb must lead in the facing direction. The knight logic is shared by both the battle-test and asset-preview scenes via `sys_knight_controls`/`sys_offsets`/`sys_attack_effects`.

# TODO
- [x] Add a `Facing` component (Left/Right) and spawn it as a child of the knight in both scenes.
- [x] Have `KnightControlSystem` write `Facing` from horizontal input and simplify walk-frame cycling.
- [x] Apply facing in the offset system: set `flip_x` on body/shield/sword and mirror shield/sword offsets when facing left.
- [x] Make attack effects mirror (offset + `flip_x`) when facing left.
- [x] Make the camera orb lead in the facing direction.
- [x] Build and verify.

# TODO Explanation

## Add a `Facing` component
- Add `pub enum Facing { Left, Right }` (derive `Clone`, `Copy`, `PartialEq`, `Eq`, `Debug`) in `src/entity/knight/components.rs` and export it from `src/entity/knight/mod.rs`.
- Create it via `HeroFactory::create_knight` (add a `facing: Facing` field to `KnightParts`, defaulting to `Facing::Right`) so both spawn sites get it for free, then add `parts.facing.into_child()` to the knight's children in both `battle_test/scene.rs::spawn_knight` and `asset_preview/scene.rs::spawn_knight`.
- Access it with `store.get_child::<Facing>(&knight)` (or `first::<Facing>()` while there is a single knight).

## Have `KnightControlSystem` write `Facing` from input
- In `update`, after computing `dir` (already -1/0/1 for left/right), if `dir != 0.0` set the knight's `Facing` to `Left`/`Right` accordingly; if `dir == 0.0`, keep the last facing so the knight stays facing its last travel direction while idle.
- Simplify walk-frame cycling: today `dir < 0.0` decrements `walk_step` (a reverse-cycle hack). Once the sprite is flipped via `flip_x`, always increment `walk_step` and let the flip handle direction, otherwise the walk animation plays backwards (moonwalk) when facing left.

## Apply facing in the offset system
- Extend `OffsetUpdateSystem` (`src/scene/asset_preview/sys_offsets.rs`) — which already reads the knight's body position + per-frame `FrameOffsets` — to also read `Facing`.
- When facing right: keep current behavior (`flip_x = false`, `child.pos = body + offset`).
- When facing left:
  - Set `body.flip_x = true`, `shield.flip_x = true`, `sword.flip_x = true`.
  - Mirror the shield/sword top-left x using `mirror_x = body_width - offset_x - child_width`, where `body_width = FRAME_SIZE` (64), `child_width = SHIELD_SIZE` (32) / `SWORD_FRAME_SIZE` (32). Y is unchanged.
  - Example: shield offset `(33,16)` mirrors to `64 - 33 - 32 = -1`; sword idle `(18,12)` -> `64 - 18 - 32 = 14`; sword thrust `(60,8)` -> `64 - 60 - 32 = -28`. This mirrors symmetrically about the body's vertical center.
- Mind store borrow rules: do not hold a read guard on the knight/body while mutating children; collect values first, then mutate (mirrors existing style in `sys_offsets.rs`).

## Make attack effects mirror when facing left
- `AttackEffectDrawSystem::draw` draws each `Effect` at `sword_pos + effect.offset` with no flip. When facing left, draw at `sword_pos + mirror_offset` and pass `flip_x: true`.
- `mirror_offset.x = sword_width - effect.offset.x - effect_texture_width` (`sword_width = SWORD_FRAME_SIZE` = 32), y unchanged. Effects are children of the `Sword`, so their offset is relative to the sword's top-left, which is already mirrored by the offset system.
- Extend the local `draw_texture` helper to take a `flip_x` param.
- The offsets are computed once at spawn (`spawn_effects`) for the right-facing case; keep them right-facing and mirror at draw time so facing changes take effect immediately.

## Make the camera orb lead in the facing direction
- `sys_orb.rs` currently sets `pos = center + vec2(100.0, 0.0)`. Change to `center + vec2(100.0 * facing_sign, 0.0)` where `facing_sign` is `+1.0` (right) / `-1.0` (left), read from `Facing`. The camera lerp already handles any target and clamps to the map.

## Build and verify
- Run `rtk cargo build --workspace 2>&1` and confirm a clean build.

# Open Questions
(All resolved: the source art faces right so a plain `flip_x` produces a correct left-facing sprite; everything mirrors — body, shield, sword, effects, orb — including `flip_x`-flipping the effect graphics.)

# Out of Scope
- We are not changing animation frame data, frame counts, or the sprite sheets.
- We are not introducing separate left-facing animation frames.
- We are not modifying the camera smoothing/lerp behavior or map clamping.
- We are not changing input bindings or movement speed.
- We are not adding facing to anything other than the knight (no enemies yet).
