# Research Topic
How the knight receives damage — tracing the full damage pipeline from enemy attack detection through to HP mutation and presentation.

## Summary

The knight's incoming-damage flow is already an **event-driven pipeline**, split across three layers:

1. **Source / detection** — an enemy system detects a hit and emits events.
2. **Resolution** — a central combat system resolves mitigation, mutates HP, and re-emits result events.
3. **Presentation** — dedicated systems react to the result events (health bar, floating text, hit reaction/knockback).

The knight is never mutated directly by the AI system. The AI only fires events; the combat system is the single place where the knight's `HeroStats.hp` is reduced.

---

## Layer 1 — Source: enemy attack detection

**File:** `src/scene/battle_test/sys_ramhead_ai.rs`

`RamHeadAiSystem` is the only system that can currently hurt the knight. Inside its `update`, during a charge state:

- It computes the charging enemy's `charge_rect` (from the `Animation` rect plus charge direction).
- It gates the hit behind `!state.hit_this_charge` so a charge deals damage **once, on first contact** (`sys_ramhead_ai.rs:411-437`).
- It hit-tests against the knight's pixel data with `did_attack(charge_rect, data)` (using `image_data_for` + the sprite sheet cache).
- On a hit it reads the enemy's `strength` from `EnemyStats` and fires **two events**:

```rust
self.bus.fire(&EnemyAttackEvent { target, damage });   // target = knight id, damage = enemy strength
self.bus.fire(&HitEvent { victim: target, attacker: *enemy_id });
```

`HitEvent` here is purely cosmetic; `EnemyAttackEvent` is the actual damage carrier.

---

## Layer 2 — Resolution: damage mitigation + HP mutation

**File:** `src/systems/sys_knight_combat.rs`

`KnightCombatSystem` subscribes to both `AttackEvent` (knight→enemy) and `EnemyAttackEvent` (enemy→knight). The knight's incoming damage is resolved in the second queue (`sys_knight_combat.rs:120-153`):

1. **Resolve the knight's combat components** in one fluent chain, so no read guard lives across the later write lock (the store `RwLock` is not reentrant):
   - `Knight` → `HeroStats` (`stats_ref`), the `armor` value, and the `Animation` `rect`.
2. **Armor buffs** — only `Consecration` is honored here:
   ```rust
   let armor = if self.store.first::<Consecration>().is_some() {
       (armor as f32 * 1.25) as i32
   } else { armor };
   ```
3. **Mitigation** via `mitigated_damage` (`sys_knight_combat.rs:12-14`):
   ```rust
   fn mitigated_damage(raw: i32, armor: i32) -> i32 { (raw - armor).max(1) }
   ```
   Flat armor reduction, floored at 1 so a successful hit always deals ≥1.
4. **Mutation**: `stats.hp = (stats.hp - damage).max(0)`.
5. **Result event**: fires `HealthChangeEvent { entity: target, amount: -damage, rect }`.

The mitigation function's own doc comment calls itself "the single point where flat mitigation (armor) and future buffs fold into the damage value".

### Mirrored path (knight → enemy)

The same system resolves `AttackEvent` for knight damage to `RamHead`s (`sys_knight_combat.rs:56-118`), including an `EnemyDeathEvent` when a target drops to 0 HP. Notably, **there is no equivalent death check for the knight** — `HeroStats.hp` can hit 0 with no event or handling.

---

## Layer 3 — Presentation (all event-driven)

| System | File | Subscribes to | Effect |
| --- | --- | --- | --- |
| `HitReactionSystem` | `src/systems/sys_hit_reaction.rs` | `HitEvent` | Hides knight visuals, shows `ImpactFrame`, applies knockback (eases `KNOCKBACK=20.0` over `DURATION=0.12s`, `2u - u²` curve). |
| `HealthBarSystem` | `src/systems/sys_health_bar.rs` | (polls `HealthBar` + stats) | Recomputes bar percent each frame. |
| `HealthTextAnimationSystem` | `src/systems/sys_health_text_animation.rs` | `HealthChangeEvent` | Floats the damage/heal number over the rect. |

All are registered on the scene's `SystemAgg` (`src/scene/battle_test/scene.rs:355-388`), with `HitReactionSystem` added at line 377 and `HealthTextAnimationSystem` at 383.

---

## Events involved (src/events.rs)

- `EnemyAttackEvent { target: u64, damage: i32 }` — the raw damage an enemy deals to the knight.
- `HealthChangeEvent { entity: u64, amount: i32, rect: Rect }` — signed; negative for damage, positive for healing.
- `HitEvent { victim: u64, attacker: u64 }` — cosmetic hit reaction trigger.
- `AttackEvent { attacker: u64, targets: Vec<u64> }` — knight→enemy only.
- `EnemyDeathEvent { enemy: u64 }` — enemy death only (no knight equivalent).

---

## Healing (the reverse pipeline)

**File:** `src/scene/battle_test/sys_divine_stance.rs:452-496`

`DivineStanceSystem::heal_knight` restores a 5% tick (`HEAL_PCT`) and fires a **positive** `HealthChangeEvent`, so healing reuses the same result-event presentation as damage. It drops every read guard before the write mutation (same non-reentrant `RwLock` constraint).

---

## Gaps / observations

1. **Evasion, accuracy, and critical are dead for the knight's defense.** `EnemyStats` and `HeroStats` both carry `evasion`, `accuracy`, `critical`, but `mitigated_damage` is the only math applied to incoming damage — no dodge/miss/crit roll happens anywhere for the knight.
2. **No invulnerability frames on dash.** `KnightDashSystem` (`src/scene/battle_test/sys_knight_dash.rs`) moves the knight but grants no i-frames; there is no `invincible`/`iframe` component anywhere.
3. **No knight death handling.** Only `EnemyDeathEvent` exists. `HeroStats.hp` can reach 0 silently.
4. **`GuardianShield` armor boost is documented but not implemented.** `src/entity/knight/components.rs:30-33` says "combat queries it to boost the knight's armor", but a grep for `GuardianShield` shows it is never read in any combat system (only `Consecration` is queried at `sys_knight_combat.rs:138`).
5. **Single flat mitigation point.** All future damage math (elemental resistance, damage types, percent reduction) is expected to fold into `mitigated_damage`, per its doc comment.
