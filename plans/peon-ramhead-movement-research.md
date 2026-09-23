# Research Topic
How the peons and ram heads move: their state machines, movement math, obstacle handling, animation, and collision interactions.

## Sources
- `src/scene/battle_test/sys_peon.rs` — peon AI + movement
- `src/scene/battle_test/sys_ramhead_ai.rs` — ram head AI + movement
- `src/entity/factory_peon.rs`, `src/entity/factory_enemy.rs` — body/collision construction
- `src/entity/peon/frames.rs`, `src/entity/enemy/frames.rs` — animation frame tables
- `src/systems/sys_collision_circle.rs` — circle/rect collision resolution
- `src/systems/sys_hit_reaction.rs`, `src/systems/sys_knight_combat.rs` — damage pipeline (referenced)

## Peon Movement (`sys_peon.rs`)

### State machine
`PeonState` enum: `Walk`, `Return`, `AttackHit1`, `AttackHit2`, `Recover`.

Per-peon runtime state (`PeonAiState`): `state`, `state_timer`, `frame_elapsed`, `walk_step`, `attack_step`, `facing_right`, `hit_this_attack`.

Transitions:
- **Walk** → `AttackHit1` when nearest enemy within `ATTACK_RANGE` (55).
- **Walk** → despawn (remove entity) when `peon_pos.x > map_w + DESPAWN_MARGIN` (80).
- **Walk** → `Return` when `peon_pos.x > knight.x + MAX_PAST_KNIGHT` (200).
- **Return** → `AttackHit1` when enemy within `ATTACK_RANGE`.
- **Return** → `Walk` when `peon_pos.x <= knight.x + RALLY_X_OFFSET` (100).
- **AttackHit1** → `AttackHit2` after `PEON_ATTACK_HIT1_FRAMES` elapse.
- **AttackHit2** → `Recover` after `PEON_ATTACK_HIT2_FRAMES` elapse.
- **Recover** → `Walk` after `RECOVER_SECS` (0.4).

### Movement math (Walk state)
1. **Chase** (`chasing = distance_to_enemy <= CHASE_RANGE` (300)): `move_dir = normalize(enemy_pos - peon_pos)`.
2. **Rally** (not chasing, knight exists): target is `vec2(knight.x + RALLY_X_OFFSET, knight.y)` where `RALLY_X_OFFSET = 100` (in front of knight). Walk toward it only when `distance > RALLY_STOP_RADIUS` (24).
3. If neither, `move_dir = None` (stand still).

Forward locomotion: `new_pos += dir * MOVE_SPEED * dt` where `MOVE_SPEED = 60.0`.

### Forward-blocking probe (recent addition)
- `forward_blocked(center, dir, self_id, peon_centers, walls)`.
- Probe point = `center + dir * (PEON_RADIUS + FORWARD_PROBE_DIST)` where `FORWARD_PROBE_DIST = 8.0`, `PEON_RADIUS = PEON_FRAME_SIZE * PEON_COLLISION_RADIUS_SCALE` (~6.08).
- Blocked if another peon's center is within `PEON_RADIUS * 2.0` of the probe, OR the probe falls inside any `CollisionRect` wall (expanded by `PEON_RADIUS`).
- When blocked, the peon holds frame `PEON_WALK_FRAMES[0]` and does **not** advance its position along `move_dir`; separation still applies (see below).

### Separation (boids-style, Walk state)
- For each other peon within `SEPARATION_RADIUS` (46): `separation += (diff/dist) * (1 - dist/SEPARATION_RADIUS)`.
- `separation_force = separation * SEPARATION_WEIGHT` (120).
- `sep_delta = separation_force * dt`; only applied when `sep_delta.length() >= SEPARATION_DEADZONE` (0.75).
- Equilibrium settled spacing ≈ `SEPARATION_RADIUS * (1 - SEPARATION_DEADZONE / 2)` ≈ 28.75 px.

### Return state movement
- Walks left at fixed `MOVE_SPEED`: `new_pos.x = peon_pos.x - MOVE_SPEED * dt`, y clamped.
- No separation, no chase, no blocking probe in `Return` — only transitions out via attack range or reaching `target_x`.

### Vertical clamp
`new_pos.y = clamp(Y_BOUND_MIN=40, Y_BOUND_MAX=360)` applied in both Walk and Return.

### Attack
- When attacking, `AttackRect.visible = true` with `rects = [attack_rect(body, facing_right)]`, a 20-px-deep strip beside the body (facing-dependent).
- `AttackEvent { attacker, targets }` fired once per attack when the attack rect overlaps a `RamHead` body.
- Attack frames use `PEON_ATTACK_HIT1_FRAMES` [7..11] then `PEON_ATTACK_HIT2_FRAMES` [12..16]; recover holds last hit frame.

### Animation
- `PEON_FRAME_COUNT = 18`; walk frames `PEON_WALK_FRAMES = [0..6]`.
- Walk frame advances when `frame_elapsed >= WALK_FRAME_DURATION` (0.12), cycling `walk_step`.
- `flip_x` in anim writes is `!facing_right` (peons face right by default; frames drawn mirrored when not facing right).

### Body / collision construction (`factory_peon.rs`)
- `PEON_FRAME_SIZE = 64.0`; `PEON_COLLISION_RADIUS_SCALE = 0.095` → collision radius ≈ 6.08.
- `CollisionGroup::Peon`.
- Body is a `64x64` Animation; collision circle center tracks the body rect center.

## Ram Head Movement (`sys_ramhead_ai.rs`)

### Design
No discrete "idle/return" state machine like peons. A ram head is an **always-forward walker** plus a brief melee attack; there is no state where the ram stops moving.

### State (`RamBrain`)
`facing` (unit dir, defaults to `(-1, 0)` left), `walk_step`, `frame_elapsed`, `attack_cooldown`, `attacking`, `attack_timer`, `attack_step`, `attack_target`.

### Movement math
- **Targeting**: `nearest_target(origin)` returns nearest living Knight or Peon (by rect-center distance). When found, `facing = normalize(target_center - center)`; else `facing = (-1, 0)`.
- **Track committed victim while attacking**: if `attacking`, keep `facing` toward the `attack_target` rect each frame.
- **Obstacle steering** (`obstacle_ahead` + `steer`):
  - `PROBE_DIST = 64`, `OBSTACLE_CLEARANCE = 32`.
  - Detects another `RamHead` ahead (along `facing`, within probe dist and lateral clearance) → `Obstacle::Ram(pos)`.
  - Detects top/bottom map boundary (within `BOUND_MARGIN = 24` of `Y_BOUND_MIN/MAX`) → `Obstacle::Boundary`.
  - `steer`: base velocity `= (facing.x * MOVE_SPEED, facing.y * MOVE_SPEED_VERTICAL)` where `MOVE_SPEED = 60`, `MOVE_SPEED_VERTICAL = 45`.
    - `Boundary`: zero out vertical component.
    - `Ram(pos)`: add perpendicular steer `perp * side * STEER_SPEED` (60), where side picks the direction away from the other ram.
- **Attack**: when `target_in_front` (forward attack rect `ATTACK_REACH = 26` deep overlaps target rect) and `attack_cooldown <= 0`, begin attack for `ATTACK_DURATION = 0.30`, set `attack_cooldown = 0.55`, fire `EnemyAttackEvent` + `HitEvent`.

### Vertical clamp
`position.y = clamp(Y_BOUND_MIN=40, Y_BOUND_MAX=360)`.

### Animation
- `RAM_HEAD_FRAME_COUNT = 8`; walk `WALK_FRAMES = [0..4]` at `FRAME_DURATION = 0.14`; attack `ATTACK_FRAMES = [5..7]` at `ATTACK_FRAME_DURATION = 0.06`.
- `flip_x = facing.x < 0.0`.

### Body / collision construction (`factory_enemy.rs`)
- `FRAME_SIZE = 64.0`; `COLLISION_RADIUS_SCALE = 0.095` → radius ≈ 6.08.
- `CollisionGroup::Enemy`.

## Collision System (`sys_collision_circle.rs`) — applies to both
- `CollisionCircle` + `CollisionRect` (walls) resolved every frame after AI systems write positions.
- `groups_interact` returns false only for `Hero <-> Peon` pairs; all other pairings (Enemy-Peon, Enemy-Hero, Enemy-Enemy, Peon-Peon) interact.
- Per-collider: axis-separated movement — try x move, revert to previous x if blocked; then y move, revert if blocked; then a 4-iteration circle-circle push-out + circle-rect push-out.
- Circle-rect uses closest-point clamp; circle-circle pushes along the axis connecting centers until `dist >= r1 + r2`.

## Key differences (peon vs ram head)
| Aspect | Peon | Ram head |
|--------|------|----------|
| Movement model | Chase/rally/return with stop states | Always-forward steer + melee |
| Idle | Stands on walk frame 0 when rally target reached or blocked | Never stops (steers around) |
| Obstacle handling | Hard stop via `forward_blocked` probe (8 px) + separation | Steering: perpendicular slide around other rams, vertical clamp at boundaries |
| Separation | Explicit boids separation force (radius 46, weight 120) | Implicit via collision push-out only |
| Target | Nearest `RamHead` only | Nearest Knight or Peon |
| Speed | 60 uniform | 60 horizontal, 45 vertical (axis-scaled) |
| Collision group | `Peon` | `Enemy` |
