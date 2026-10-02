# Ram Head Attacking — How It Currently Works

Research into how a Ram Head enemy performs its melee attack. All logic lives in
`src/scene/battle_test/sys_ramhead_ai.rs` (`RamHeadAiSystem`); the attack is a
small per-ram state machine plus a cosmetic animation loop. There is **no hit
detection against the ram's `AttackRect`** — damage is fired as an event the
instant the attack starts.

## Where the state lives

Per-ram brain state is a `RamBrain` stored in `states: HashMap<u64, RamBrain>`,
keyed by the ram entity id (`sys_ramhead_ai.rs:112`). Attack-relevant fields
(`sys_ramhead_ai.rs:59-73`):

- `attacking: bool` — true while a melee attack is active (opens the `AttackRect`)
- `attack_timer: f32` — remaining time of the active attack
- `attack_step: usize` — index into `ATTACK_FRAMES`
- `attack_cooldown: f32` — seconds until the next attack is allowed
- `attack_target: Option<u64>` — the hero id the ram committed to
- `frame_elapsed: f32` — animation accumulator (shared by walk/attack)

The brain is cloned out at the top of the loop, mutated, and written back with
`self.states.insert(*enemy_id, brain)` (`sys_ramhead_ai.rs:245`, `:462`).

## Timing constants

```
ATTACK_REACH         = 26.0   // depth of the forward hit strip
ATTACK_DURATION      = 0.30   // how long the attack stays active
ATTACK_FRAME_DURATION= 0.06   // per attack-animation frame
ATTACK_COOLDOWN      = 0.55   // before the same ram can attack again
```
(`sys_ramhead_ai.rs:17-20`)

Animation frames come from `src/entity/enemy/frames.rs`:
`WALK_FRAMES = [0,1,2,3,4]`, `ATTACK_FRAMES = [5,6,7]`, sheet has 8 frames.

## Trigger: a plain rect overlap

Each frame `can_attack` is true when the ram is in `Stalk` or `Charge`, its
`attack_cooldown` has elapsed, and it is not already attacking
(`sys_ramhead_ai.rs:371-373`). It then calls `target_in_front`, which is a simple
`Rect::overlaps` test between the hero's `Animation::rect()` and
`forward_rect(body, facing)` (`sys_ramhead_ai.rs:210-218`).

`forward_rect` builds a 26px-deep strip attached to the front face of the ram's
body along its dominant axis (horizontal or vertical, whichever `facing` points
along), so the strip sits ahead of the ram rather than over its own body
(`sys_ramhead_ai.rs:119-137`). This is a coarse rect test — **no per-pixel
`did_attack` sampling** is done for ram heads (unlike the knight, see below).

## Damage is event-driven, fired at attack start

The moment the trigger succeeds, in the *same frame*, the ram sets
`attacking = true`, `attack_timer = ATTACK_DURATION`, `attack_step = 0`,
`attack_cooldown = ATTACK_COOLDOWN`, `attack_target = Some(target_id)`
(`sys_ramhead_ai.rs:376-382`), then immediately fires two events
(`sys_ramhead_ai.rs:384-399`):

```rust
self.bus.fire(&EnemyAttackEvent { target: target_id, damage: strength });
self.bus.fire(&HitEvent { victim: target_id, attacker: *enemy_id });
```

`damage` is the ram's `EnemyStats::strength` (default 18, read from the
`EnemyStats` child; `sys_ramhead_ai.rs:384-390`). `EnemyAttackEvent` is consumed
by `KnightCombatSystem`, which applies armor mitigation `(raw - armor).max(1)`,
decrements `HeroStats::hp`, and fires death/health-change events
(`src/systems/sys_knight_combat.rs:122-184`). `HitEvent` drives the knockback /
impact-frame reaction (`src/systems/sys_hit_reaction.rs`).

Consequences:

- Damage is **instant and guaranteed** to the target found by the rect test; it
  is not re-validated later, so a target that leaves during the swing still gets
  hit.
- Damage is dealt on the **first frame** of the swing (before the animation
  visually connects). There is no per-frame damage window and no windup on the
  melee itself (the charge windup is a separate mode).

## The animation is cosmetic and loops

While `attacking`, the frame is picked from `ATTACK_FRAMES` and advanced with a
modulo, so it wraps back to frame 5 (`sys_ramhead_ai.rs:416-423`):

```rust
let frame = ATTACK_FRAMES[brain.attack_step];
brain.frame_elapsed += dt;
if brain.frame_elapsed >= ATTACK_FRAME_DURATION {
    brain.frame_elapsed = 0.0;
    brain.attack_step = (brain.attack_step + 1) % ATTACK_FRAMES.len();
}
```

With `ATTACK_DURATION = 0.30` and `ATTACK_FRAME_DURATION = 0.06` there are
exactly 5 advances over the attack, cycling the 3-frame sequence as
`5,6,7,5,6` — i.e. the swing animation **loops** for the duration rather than
playing once. When not attacking, frames come from `WALK_FRAMES` (moving) or
hold `WALK_FRAMES[0]` (nearly stopped) (`sys_ramhead_ai.rs:424-434`).

The attack ends when `attack_timer` ticks to 0, which clears `attacking` and
`attack_target` (`sys_ramhead_ai.rs:284-291`).

## The AttackRect is written but never used for damage

Each frame, while attacking, the system writes the ram's `AttackRect` child:
`rects = vec![forward_rect(body, facing)]`, `visible = true`; otherwise empty /
false (`sys_ramhead_ai.rs:444-460`). The rect is spawned inert in the spawner
(`sys_ramhead_spawner.rs:63-67`).

However, **no system reads a Ram Head's `AttackRect` for hit detection**. The
only reader of enemy `AttackRect`s is `DebugDrawSystem`, which just outlines all
visible `AttackRect`s in yellow (`src/systems/sys_debug_draw.rs:40-47`). So for
ram heads the component is effectively debug/cosmetic; actual damage bypasses it.

This is the opposite of the knight, whose `AttackRect` *is* the source of truth:
`KnightAttackHitSystem` runs `image_data_for` + `did_attack` against it and only
fires `AttackEvent` on the rising edge of `area.visible`
(`src/scene/battle_test/sys_knight_attack_hit.rs:59-75`). Per
`rules/attacks.md`, all damage should go through `AttackRect`, so the ram head is
currently an exception to that rule.

## Attack ↔ movement-mode interaction

- Attack is allowed in `Stalk` (adjacent prey) or mid-`Charge` (the charge
  connects) (`sys_ramhead_ai.rs:371`).
- If the attack lands while in `Charge`, the mode is forced to `Recover` and
  `charge_cooldown` is set (`sys_ramhead_ai.rs:401-405`).
- While attacking, `facing` keeps re-aiming at `attack_target`'s current rect, so
  the visual body (and the forward `AttackRect`) track the committed victim even
  though damage already landed (`sys_ramhead_ai.rs:266-279`).

## Notes / possible inconsistencies

- Damage fires before any animation frame is shown and is unconditional once the
  rect tests true — there is no "active frames" damage window.
- `ATTACK_DURATION / ATTACK_FRAME_DURATION = 5` ticks but the animation is only 3
  frames and is modulo-looped, so frames 5 and 6 each appear twice and 7 once. If
  a one-shot swing were intended, the step should clamp rather than wrap.
- The `AttackRect` on a Ram Head has no consumer, so it duplicates the
  `forward_rect` computation that is already used directly by `target_in_front`.
- `target_in_front` uses rect overlap only; the damage system trusts the target
  id and applies damage with no `did_attack` pixel check.

## Related files

- `src/scene/battle_test/sys_ramhead_ai.rs` — all attack logic
- `src/scene/battle_test/sys_ramhead_spawner.rs` — spawns the inert `AttackRect`
- `src/entity/enemy/frames.rs` — `WALK_FRAMES` / `ATTACK_FRAMES`
- `src/entity/enemy/components.rs` — `RamHead`, `EnemyStats`
- `src/entity/factory_enemy.rs` — `create_ram_head` (64px frames, `frame_duration` 0.12)
- `src/systems/sys_knight_combat.rs` — applies `EnemyAttackEvent` damage
- `src/systems/sys_hit_reaction.rs` — knockback / impact on `HitEvent`
- `src/systems/sys_debug_draw.rs` — only consumer of the ram's `AttackRect`
- `src/scene/battle_test/sys_knight_attack_hit.rs` — knight's AttackRect-based attack (contrast)
