# Description
Add a new knight skill, "Shield Cycle". On cast, the knight enters the swipe pose (`SWIPE_FRAME`) and throws a circular disc that flies forward toward the knight's facing. When the disc detects an enemy, it carries forward an extra 30px in its flight direction, stops, and grinds there, dealing 5 discrete damage rounds (10 flat damage each) to every enemy overlapping its attack rect.

All original open questions are resolved below; answers are marked **Resolved:** in TODO Explanation and reflected in Objects/Services.

# TODO
- [ ] Add a `ShieldCycle` skill icon kind and wire it into the Left selector slot
- [ ] Create the `ShieldCycleSystem` with fly → detect → glide → grind phases
- [ ] Register the system in the battle scene and remove the Left guard in `SkillSystem::fire`

# TODO Explanation
## Add a `ShieldCycle` skill icon kind and wire it into the Left selector slot
Add a `ShieldCycle` variant to `SkillIconKind`. Map it to the existing shield icon texture in `skill_texture` (no new asset) so it renders in the bar and selector instead of a procedural glyph. In `BattleTestScene::spawn_skills_widget`, replace `SkillSlotCfg::empty().direction(SkillDirection::Left)` with `SkillSlotCfg::icon(SkillIconKind::ShieldCycle).direction(SkillDirection::Left)`.

**Resolved — slot:** Shield Cycle takes the Left slot, the current empty placeholder (`sys_skill.rs` comments it as "the empty placeholder slot: nothing to cast"), leaving Sword=Up, Guardian Shield=Right, Fireball placeholder=Down untouched.

**Resolved — icon:** reuse the existing shield icon texture (`icons.shield` / `images::SkillIcon::Shield`); no new icon asset.

## Create the `ShieldCycleSystem` with fly → detect → glide → grind phases
Model after `SwordsSkillSystem` / `GuardianShieldSystem`: subscribe to `SkillCastEvent` filtered on `kind == ShieldCycle`; on cast, set the knight to `SWIPE_FRAME`, add a `KnightLock`, and spawn a disc as a standalone `Animation` entity carrying an inert `AttackRect` child (same spawn shape as the swords). Each frame the disc flies along its facing, updating its `AttackRect` to its own footprint and hit-testing enemies via `did_attack(rect, &image_data_for(&body, &mut self.sheet_cache))`; on a hit it enters glide, carries 30 more px, then stops and grinds. Each grind round re-hit-tests the rect and damages every overlapping enemy. After 5 rounds the disc despawns.

**Resolved — "5 rounds":** 5 discrete damage ticks at a fixed interval. The first tick fires immediately when grinding begins, then one tick per `DISC_TICK_SECS` for the remaining 4, each applying 10 flat damage.

**Resolved — targets per round:** every enemy overlapping the `AttackRect` that round — per the user's own wording "the enemy (or enemies in the attackrect)".

**Resolved — damage model:** flat 10 damage applied directly to `EnemyStats.hp`, consistent with `SwordsSkillSystem` (no armor mitigation; Ram Head armor is 0 anyway). Each round mirrors the swords damage flow: clamp hp, fire `EnemyDeathEvent` if killed, `HealthChangeEvent` with the enemy rect, `HitEvent` with the caster id.

**Resolved — disc sprite:** reuse the knight's carried shield texture, resolved in `new()` exactly like `GuardianShieldSystem` (`store.first::<Shield>` → `StaticImage.source`, falling back to `assets.texture(images::Knight::Shield1)`); no new asset.

Extra decided details:
- Disc flight is horizontal along the knight's `Facing`, matching the swords (`Facing::Left => -1.0`, `Right => 1.0`).
- The knight is held in `SWIPE_FRAME` + `KnightLock` for a short fixed stance, then released while the disc continues autonomously — same pattern as `GuardianShieldSystem` (0.25s stance) and `SwordsSkillSystem` (0.5s lock). Proposed `STANCE_SECS = 0.3`.
- If the disc detects nothing it despawns at `DISC_RANGE` from spawn (swords use the 400px view width).
- Disc drawn at native 32x32 (shield texture), `z_idx 4.0`, `current_frame 0`, `frame_count 1`.
- Constants: `DISC_DAMAGE = 10`, `DISC_ROUNDS = 5`, `DISC_TICK_SECS = 0.5`, `DISC_GLIDE = 30.0`, `DISC_SPEED = 600.0`, `DISC_RANGE = 400.0`, `STANCE_SECS = 0.3`.

## Register the system in the battle scene and remove the Left guard in `SkillSystem::fire`
Add `ShieldCycleSystem` to the `SystemAgg` in `BattleTestScene::load` next to `SwordsSkillSystem` / `GuardianShieldSystem`. Remove the `SkillDirection::Left` early-return in `SkillSystem::fire` so the Left slot can cast.

# Objects

## SkillIconKind (modify)
Adds a `ShieldCycle` variant so the skill has a selector identity; file `src/entity/skills/components.rs`.

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkillIconKind {
    Sword,
    Shield,
    Potion,
    Fireball,
    Crossed,
    ShieldCycle, // added
}
```

## DiscPhase (new)
Per-disc life phase driving the fly → glide → grind behaviour; lives in `src/scene/battle_test/sys_shield_cycle.rs`.

```rust
enum DiscPhase {
    Flying,   // travelling until an enemy is detected
    Gliding,  // carrying forward the extra 30px after detection
    Grinding, // stationary, dealing damage each round
}
```

## ShieldDisc (new)
Per-disc runtime state held by the system while the cast is active; lives in `src/scene/battle_test/sys_shield_cycle.rs`.

```rust
struct ShieldDisc {
    anim_ref: EntityRef,  // the disc's standalone Animation entity
    area_ref: EntityRef,  // the disc's AttackRect child entity
    dir: Vec2,            // unit flight direction derived from the knight's facing
    phase: DiscPhase,     // fly / glide / grind
    glide_remaining: f32, // px still to travel after detection (starts at DISC_GLIDE)
    rounds_left: u32,     // damage rounds remaining (starts at DISC_ROUNDS)
    tick_timer: f32,      // time until the next damage round fires
}
```

## ShieldCycleSystem (new)
The system's state; lives in `src/scene/battle_test/sys_shield_cycle.rs`, registered in the battle scene.

```rust
pub struct ShieldCycleSystem {
    store: Rc<EStore>,                              // entity store
    bus: Rc<EventBus>,                              // used to fire damage/death events
    queue: Rc<RefCell<VecDeque<SkillCastEvent>>>,   // cast events captured by the subscription
    _subs: Rc<SubCollection>,                       // keeps the SkillCastEvent subscription alive
    disc: Option<ShieldDisc>,                       // the active disc, if a cast is live
    caster: u64,                                    // knight entity id from the cast event
    stance_remaining: f32,                          // time left in the swipe-pose lock
    disc_tex: Texture2D,                            // knight shield texture, resolved in new()
    sheet_cache: Option<(Texture2D, Image)>,        // enemy pixel cache for did_attack
}
```

# Services

## ShieldCycleSystem (new)
Update/draw system for the skill; file `src/scene/battle_test/sys_shield_cycle.rs`.

```rust
impl ShieldCycleSystem {
    /// Builds the system, subscribing its queue to `SkillCastEvent` and resolving the disc texture.
    pub fn new(store: Rc<EStore>, bus: Rc<EventBus>, assets: &Assets) -> Self { ... }

    /// Starts a cast: filters kind == ShieldCycle, enters the swipe pose, locks the knight, spawns the disc.
    fn begin_cast(&mut self, event: &SkillCastEvent) { ... }

    /// Advances the disc through fly / glide / grind and applies damage rounds until it despawns.
    fn update_cast(&mut self, dt: f32) { ... }

    /// Re-hit-tests the disc's AttackRect and applies 10 damage to every enemy it overlaps.
    fn apply_round(&mut self) { ... }

    /// Removes the `KnightLock` and returns the knight to `IDLE_FRAME`.
    fn release_lock(&mut self) { ... }
}

impl System for ShieldCycleSystem {
    /// Drains queued casts, ticks the stance lock, then advances the active disc.
    fn update(&mut self, ctx: &mut Context) { ... }

    /// Draws the disc sprite while a cast is active.
    fn draw(&self, _ctx: &Context) { ... }
}
```

## SkillSystem (modify)
Removes the Left-slot early-return so the new skill can cast; file `src/scene/battle_test/sys_skill.rs`.

```rust
impl SkillSystem {
    // ... existing methods ...

    /// Fires the skill bound to `dir` — changed: the `SkillDirection::Left`
    /// "empty placeholder" early-return is removed so Left can cast Shield Cycle.
    fn fire(&mut self, dir: SkillDirection) { ... }
}
```

## skill_texture (modify)
Maps the new icon kind to the existing shield texture; file `src/ui/skill_icons.rs`.

```rust
/// Maps an icon kind to its backing texture — changed: `ShieldCycle` now returns `Some(&icons.shield)`.
fn skill_texture(kind: SkillIconKind, icons: &SkillIconTextures) -> Option<&Texture2D> { ... }
```

# Open Questions
None — all resolved above.

# Out of Scope
- No new damage model: 10 flat damage per round, no armor mitigation.
- No new icon or disc sprite assets (reuse the knight shield texture for both).
- No VFX beyond drawing the disc sprite; the shared golden-magic shader pass is optional polish.
- No changes to the other skills (Swords, Guardian Shield, Fireball placeholder).
