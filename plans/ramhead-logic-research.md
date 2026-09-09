# RamHead Logic

How the RamHead enemy works end to end: entity construction, animation constants, AI (move/attack state machine), and the full "get hit" pipeline (sword overlap detection → event → damage/flash/floating text → health bar). RamHead is the only enemy currently in the game.

## Entity structure

- Marker component: `RamHead` (empty struct) in `src/entity/enemy/components.rs`. Re-exported from `src/entity/enemy/mod.rs`.
- Companion component `EnemyStats` holds flat, non-leveling combat stats. Defaults: name "Ram Head", hp 60, max_hp 60, strength 8, armor 0, evasion 5, accuracy 90, critical 5 (`src/entity/enemy/components.rs:18`).
- Spawned by `spawn_ram_head` in `src/scene/battle_test/scene.rs:165`. The factory builds three parts and the store parents them under the `RamHead` marker:
  - `RamHead` (root marker)
  - child `Animation` (the sprite)
  - child `CollisionRect`
  - child `HealthBar` (`HealthBar::default()`)
  - child `EnemyStats` (`EnemyStats::default()`)

```
RamHead
├── Animation      (sprite, 64x64, 8 frames)
├── CollisionRect  (32x22 rect, currently unused by any system)
├── HealthBar      (bar UI, positioned by HealthBarSystem)
└── EnemyStats     (hp/max_hp/strength/...)
```

Only one RamHead is spawned in the test scene, at map center: `spawn_ram_head(vec2(map_w * 0.5, map_h * 0.5))` (`scene.rs:332`).

## Factory & sprite constants

- `EnemyFactory::create_ram_head` (`src/entity/factory_enemy.rs:24`):
  - Texture: `images/Enemy::RamHead` → `"images/enemies/anim-ram-head.png"` (`src/access/ids.rs:181`).
  - `FRAME_SIZE = 64.0`; frame_count = `RAM_HEAD_FRAME_COUNT = 8` (`src/entity/enemy/frames.rs:1`).
  - Animation starts at frame 0 with `running: false`, scale 1.0, visible.
  - Collision rect is `64*0.5` x `64*0.35` = 32 x 22 px, positioned at origin (set later by the scene only for `Animation`; the `CollisionRect`'s rect is never repositioned in code, and no system consumes it).
- Sprite sheet layout (`src/entity/enemy/frames.rs`): 8 frames across one row.
  - `IDLE_FRAME = 0`
  - `WALK_FRAMES = [0,1,2,3,4]` (frames 0-4)
  - `ATTACK_FRAMES = [5,6,7]` (frames 5-7)
- Movement constants (`src/entity/enemy/movement.rs`): `MOVE_SPEED = 62.5`, `MOVE_SPEED_VERTICAL = 62.5*0.75`, `FRAME_DURATION = 0.14s` per frame, `ATTACK_RANGE = 72.0` px.

## AI state machine (`src/scene/battle_test/sys_enemy_ai.rs`)

Registered once in `BattleTestScene::new` (`scene.rs:60`). State is stored on the system struct, not in the store: `attacking: bool`, plus frame-elapsed timer and walk/attack step counters.

Per frame it:
1. Finds the first `RamHead` → its child `Animation` (note: `.first::<RamHead>()`, so it drives only the first enemy — single-enemy assumption).
2. Reads the enemy position and the player Knight's `Animation` position (`PlayerOne → Knight → Animation`).
3. If there is no Knight, sets the enemy back to `IDLE_FRAME` and stops.
4. Computes `delta = knight_pos - enemy_pos`, `distance`, `in_range = distance <= ATTACK_RANGE` (72 px).
5. If `in_range` changed, toggles `attacking` and resets timers/step counters.
6. Movement: only when **not** attacking — normalizes the vector to the Knight and steps by `(MOVE_SPEED, MOVE_SPEED_VERTICAL)` per second, applied with `ctx.dt`. Vertical axis moves slower (top-down perspective illusion). Positions are `.round()`ed to keep pixel-aligned.
7. Animation frame: `next_frame(dt)` accumulates `frame_elapsed`; every `FRAME_DURATION` (0.14s) it advances `walk_step` through `WALK_FRAMES` (cycle of 5) or `attack_step` through `ATTACK_FRAMES` (cycle of 3). If still inside the frame duration, it holds the current step's frame (steady pacing regardless of dt).
8. Writes position, `current_frame`, and `flip_x` (mirrored when the knight is to the left) back to the Animation.

Because the RamHead Animation is created with `running: false`, the generic `AnimationUpdateSystem` (`src/scene/asset_preview/sys_animation.rs`, which only advances `running` animations) never touches it — the enemy's frames are driven exclusively by this AI system.

**Important nuance**: "attacking" is purely visual. When in range, the enemy stops and loops the attack frames (5→6→7) but deals **no damage** — nothing reads `EnemyStats.strength`, and there is no enemy→knight damage resolution anywhere. See Gaps below.

## Getting hit (the attack pipeline)

Three systems cooperate: a detector, a handler, and a UI updater.

### 1. Overlap detection — `AttackHitSystem` (`src/scene/battle_test/sys_attack_hit.rs`)

Registered in `scene.rs:65`. Each frame it:
- Locates the Knight → Sword → `AttackArea` component.
- Iterates every `RamHead` and checks whether the enemy's `Animation.rect()` overlaps **any** rect in the sword's `AttackArea.rects`.
- The `AttackArea` is fed by `KnightAttackSystem` (`src/scene/asset_preview/sys_knight_attack.rs`), which rebuilds the attack rects each frame from the sword rect (trimmed to a lower band: `r.y += h*0.3; r.h *= 0.4`) plus the active thrust/slash effect rect (75% of effect texture), and sets `area.visible = true` only on the sword's `THRUST_FRAME`/`SWIPE_FRAME`.
- Fires a single `AttackEvent { attacker: knight_id, targets: Vec<ids> }` on the **rising edge** — `area.visible && !prev_visible` — so overlapping for the whole visible attack window doesn't re-fire every frame. `targets` is capped to enemies actually overlapped that frame.
- Tracks `prev_visible` locally on the system struct.

### 2. Hit resolution — `HandleAttackSystem` (`src/systems/sys_handle_attack.rs`)

Registered as both update and draw (`scene.rs:306-309`); it is a `#[derive(Clone)]` system, so the event queue, flash map, etc. live behind `Rc<RefCell<…>>`.

- Subscribes to `AttackEvent` in `new()` (queue-only handler, per events rules); drains in `update()`.
- Damage = attacker Knight's `HeroStats.strength` (via `get_by_id::<Knight>(attacker)` → child `HeroStats`), defaulting to 1, clamped `.max(1)`.
- For each target id:
  - Records a white-flash entry `flashing[target] = FLASH_DURATION (0.15s)`.
  - Resolves target → child `EnemyStats` and reduces `hp` by damage, `.max(0)` (no overkill below zero; **no death handling**).
  - Resolves the target's `Animation` rect and fires `HealthChangeEvent { entity, amount: -damage, rect }` — consumed by the floating damage text system (see `plans/ramhead-hit-text-research.md`).
- Each frame it also decays `flashing` timers (retains entries > 0).
- In its `Draw` pass, while any enemy is flashing it binds `FlashWhiteFrag` material (compiled once in `load_flash_material`, `sys_handle_attack.rs:45`) and re-draws each flashing enemy's current animation frame — the brief white "hit" flash overlay.

### 3. Health display — `HealthBarSystem` (`src/systems/sys_health_bar.rs`)

Registered in `scene.rs:68` as an update system (recomputes geometry each frame; the actual bar pixels are presumably drawn by a drawable on `HealthBar`).
- For every `HealthBar`, finds its parent and requires the parent to be a `RamHead`.
- Pulls the parent's `Animation` rect and `EnemyStats` → `percent = hp/max_hp` (clamped to [0,1]).
- Recomputes the bar position centered above the sprite: `x = rect.x + (rect.w - width)*0.5`, `y = rect.y - height`, and sets `z_idx = anim.z_idx + 0.5` so the bar draws above the enemy.
- Writes are deferred to a scratch `Vec` and applied after the iteration loop (avoids borrow conflicts).

Note: damage text/flash are produced in the same `HandleAttackSystem` that applies damage, not by the hit-detection system; the RamHead itself carries no damage logic — everything that happens *to* it is driven by the systems above.

## Events used

All defined in `src/events.rs`:
- `AttackEvent { attacker: u64, targets: Vec<u64> }` — fired by `AttackHitSystem`, handled by `HandleAttackSystem`.
- `HealthChangeEvent { entity: u64, amount: i32, rect: Rect }` — fired by `HandleAttackSystem`, handled by `HealthTextAnimationSystem` (the floating numbers).

The RamHead is referenced by raw `u64` entity id in every event; receivers re-resolve the id to the `RamHead` → child component chain.

## Key files

- `src/entity/enemy/components.rs` — `RamHead` marker, `EnemyStats`.
- `src/entity/enemy/frames.rs` — frame counts / walk / attack frame sets.
- `src/entity/enemy/movement.rs` — speed & range constants.
- `src/entity/factory_enemy.rs` — builds the sprite + collision rect.
- `src/scene/battle_test/sys_enemy_ai.rs` — movement + attack-animation state machine.
- `src/scene/battle_test/sys_attack_hit.rs` — sword-overlap detection, fires `AttackEvent`.
- `src/scene/asset_preview/sys_knight_attack.rs` — builds `AttackArea` rects + visibility.
- `src/systems/sys_handle_attack.rs` — applies damage, white flash, fires `HealthChangeEvent`.
- `src/systems/sys_health_bar.rs` — repositions health bar above the sprite.
- `src/scene/battle_test/scene.rs` — spawns the enemy and registers all systems.

## Gaps / observations

- **Enemy never actually attacks.** In range it only plays the attack animation; `EnemyStats.strength` (and armor/evasion/accuracy/critical/name) are never read anywhere in game code. The knight has no health/defense pipeline that the enemy can feed.
- **No death/respawn handling.** HP clamps at 0 but the RamHead stays alive and mobile; nothing despawns or removes it, and AI ignores hp entirely.
- **Single-enemy logic.** `EnemyAiSystem` uses `.first::<RamHead>()`, and the whole `battle_test` scene spawns just one, so multi-enemy behaviour is untested (attack/hit code does iterate `all::<RamHead>()`).
- **AI state is per-system, not per-entity.** Move/attack step counters and the `attacking` flag live on `EnemyAiSystem`, so two RamHeads would share/contend for one set of animation state.
- **`CollisionRect` on the enemy is inert** — the factory makes one (32x22) but no system positions or reads it (map/solid collision appears unhandled for enemies; AI moves freely).
- Vertical move speed is slower than horizontal by design (75%).
