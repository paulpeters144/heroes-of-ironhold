# Description
Add a new visual-only skill, Guardian Shield, to the battle test scene. On cast the knight holds the swipe pose (`SWIPE_FRAME`, frame 2) for 500ms, then a shield sprite — the pre-made texture `assets/images/knight/static-knight-lg-shield.png` (64x64) — appears directly in front of the knight. No procedural shapes and no shader material: the sprite is drawn with `draw_texture_ex` and the fade in/out is just alpha in the draw color. No gameplay effect, damage, or blocking yet — this pass is about getting the visuals right.

# TODO
- [ ] Register the shield image in `src/access/ids.rs` and preload it in `scene.rs`
- [ ] Create `sys_guardian_shield.rs` with the `GuardianShieldSystem`
- [ ] Register the system in `src/scene/battle_test/mod.rs` and `scene.rs`
- [ ] Verify the cast reaches the system (SkillSystem already fires `SkillCastEvent` for the Shield slot)
- [ ] Build and visually verify

# TODO Explanation
## Register the shield image
Add a `LgShield` variant to the `images::Knight` enum in `src/access/ids.rs` with path `"images/knight/static-knight-lg-shield.png"` (alongside the existing `ThrustGraphic` / `SwipeGraphic` static sprites). The file is already in `assets/images/knight/`. Two changes are required, not one: (1) the `Id` variant above, and (2) add `&images::Knight::LgShield` to the `assets.preload(&[...])` list in `BattleTestScene::load()` (`scene.rs`). `Assets::texture()` panics on any id that was not preloaded, so omitting step (2) crashes the first cast.

## Create `sys_guardian_shield.rs`
A new system mirroring `SwordsSkillSystem`'s shape: subscribes to `SkillCastEvent`, ignores kinds other than `SkillIconKind::Shield`, puts the knight into the swipe pose with a `KnightLock`, counts down the 500ms stance, then renders the shield sprite in front of the knight. The shield is pure graphics — drawn by the system's `draw()`, not a store entity, and carries no `AttackRect` or gameplay data. The texture is loaded once in `new()` via `assets.texture(images::Knight::LgShield)` and stored on the struct (never call `Assets` accessors in `update()`/`draw()`).

## Register the system
Add `mod sys_guardian_shield;` to `src/scene/battle_test/mod.rs` and `use super::sys_guardian_shield::GuardianShieldSystem;` to `scene.rs`, then construct it in `BattleTestScene::load()` right after `SwordsSkillSystem`: `GuardianShieldSystem::new(self.store.clone(), self.bus.clone(), &self.assets)`. A single `agg.add(...)` registers it — `SystemAgg` has one list and calls every system's `update`/`draw`/`draw_ui` (default no-ops), so one `add` covers both phases. Placement after `DrawSystem` is what makes the shield render on top of the knight: the aggregate draws in insertion order. (Event delivery does not depend on registration order — the bus notifies every subscriber; the queue is drained on the system's next `update`.)

## Verify the cast reaches the system
The Shield slot is already bound to `SkillDirection::Right` in `spawn_skills_widget()` and fires a `SkillCastEvent` with `kind: SkillIconKind::Shield` — no change to `SkillSystem` needed. Confirm the event lands on the new subscription.

## Build and visually verify
`rtk cargo build --workspace --all-targets` must be clean. In-game: cast the skill, confirm 500ms of swipe pose, then the shield sprite in front of the knight.

## Shield tracking after release
Once the STANCE_SECS (500ms) stance ends the KnightLock is released and the shield becomes active. The shield follows the knight thereafter: each frame `draw_shield` re-reads the knight's current center and `Facing`, so the shield stays in front of the knight even if it turns around. The shield sits `SHIELD_AHEAD` (20px) in front of the knight's center and is drawn at `GUARDIAN_SHIELD_SIZE` (64px) square — the PNG is exactly 64x64, so this is a 1:1 native-size draw with no scaling. The shield deactivates after `SHIELD_SECS` (15 seconds, named constant) of being visible. Alpha fades in over `FADE_IN_SECS` (250ms) at spawn and fades out over `FADE_OUT_SECS` (250ms) before despawn, applied through the draw color's alpha. Nothing else cancels the shield: Swords casts, attacks, and movement leave it up, and a second Guardian Shield cast is ignored while one is already active (only one shield at a time).

# Objects

## GuardianShieldSystem (new)
The system struct that owns the skill's lifecycle state. Lives in `src/scene/battle_test/sys_guardian_shield.rs`. The shield's visible lifetime is a named constant so it is trivial to tune:

```rust
/// How long the knight is held in the swipe pose before the shield appears
/// (seconds) — mirrors SwordsSkillSystem's LOCK_SECS.
const STANCE_SECS: f32 = 0.5;
/// How long the shield stays visible after it appears (seconds).
const SHIELD_SECS: f32 = 15.0;
/// Time (seconds) the shield spends fading in after it appears.
const FADE_IN_SECS: f32 = 0.25;
/// Time (seconds) the shield spends fading out before it despawns.
const FADE_OUT_SECS: f32 = 0.25;
/// How far in front of the knight's center the shield sits (world units).
const SHIELD_AHEAD: f32 = 20.0;
/// Guardian shield footprint: matches both the knight's frame (factory_hero
/// FRAME_SIZE, 64) and the PNG's native 64x64, so it draws unscaled. Named
/// GUARDIAN_SHIELD_SIZE, not SHIELD_SIZE, to avoid clashing with
/// factory_hero::SHIELD_SIZE (32.0, the knight's own carried shield).
const GUARDIAN_SHIELD_SIZE: f32 = 64.0;
```

```rust
pub struct GuardianShieldSystem {
    store: Rc<EStore>,                          // store access for knight pose/lock lookups
    queue: Rc<RefCell<VecDeque<SkillCastEvent>>>, // queued cast events drained in update()
    _subs: Rc<SubCollection>,                   // keeps the SkillCastEvent subscription alive
    stance_remaining: f32,                      // seconds left in the swipe pose before the shield appears
    shield_active: bool,                        // true once the shield graphic is shown
    shield_life: f32,                           // seconds since the shield appeared; despawns at SHIELD_SECS
    shield_tex: Texture2D,                      // the shield sprite, loaded once in new()
}
```

Unlike `SwordsSkillSystem` this struct deliberately has no `bus` or `caster` fields: the system fires no events back (no gameplay effect), so nothing reads them after construction and they would trip the "field is never read" warning against the clean-build requirement. `new()` still takes `bus` to create the subscription, and the knight is resolved via the `PlayerOne` → `Knight` chain (or `event.caster`, used transiently inside `begin_cast`).

Tracking does not store a direction: `shield_position` re-reads the knight's `Facing` from the store each frame, so the shield follows the knight without any cast-time snapshot.

# Services

## GuardianShieldSystem (new)
System lifecycle and behavior contract. Lives in `src/scene/battle_test/sys_guardian_shield.rs`.

```rust
impl GuardianShieldSystem {
    /// Subscribes to SkillCastEvent and loads the shield texture
    /// (images::Knight::LgShield) onto the struct; returns the ready system.
    pub fn new(store: Rc<EStore>, bus: Rc<EventBus>, assets: &Assets) -> Self { ... }

    /// Resolves the knight's facing and animation ref, sets the swipe pose, adds a
    /// KnightLock (by "GuardianShield"), and starts the STANCE_SECS timer.
    /// Ignored entirely if a shield is already active or a stance is already
    /// running — only one shield at a time. Also defensively ignored if the
    /// knight already carries a KnightLock (SkillSystem gates this anyway,
    /// mirroring SwordsSkillSystem). Resets stance/shield state for the new cast.
    fn begin_cast(&mut self, event: &SkillCastEvent) { ... }

    /// Sets the knight's animation to SWIPE_FRAME (frame 2, the swipe pose —
    /// which also hides the knight's own sword and shield). Called from begin_cast.
    fn enter_swipe_pose(&mut self, anim_ref: &EntityRef) { ... }

    /// Returns where the shield should be drawn: the knight's current center plus the
    /// current facing direction times SHIELD_AHEAD. Re-read each frame so the
    /// shield tracks the knight and stays in front even if it turns around.
    fn shield_position(&self, knight_center: Vec2, facing: Facing) -> Vec2 { ... }

    /// Draws the shield sprite in front of the knight when shield_active is true:
    /// a plain draw_texture_ex of shield_tex at GUARDIAN_SHIELD_SIZE square (1:1 with the
    /// 64x64 PNG), top-left = shield_position - GUARDIAN_SHIELD_SIZE/2. Position re-resolves
    /// the knight's current center and facing each frame so the shield tracks the
    /// knight. The fade is alpha in the draw color:
    /// Color::new(1.0, 1.0, 1.0, alpha) — ramping up over FADE_IN_SECS at spawn and
    /// down over FADE_OUT_SECS before despawn. No material: macroquad's default
    /// pipeline alpha-blends textured draws, so the fade just works.
    fn draw_shield(&self) { ... }
}

impl System for GuardianShieldSystem {
    /// Drains queued casts; decrements stance_remaining; when it reaches 0, activates
    /// the shield graphic and releases the KnightLock so the knight is free again.
    /// Releasing mirrors SwordsSkillSystem::release_lock: remove the KnightLock
    /// child and restore the knight's animation to IDLE_FRAME (the control system
    /// resumes writing frames on its next update regardless).
    /// While active, tracks shield_life and deactivates the shield after SHIELD_SECS.
    /// Other actions (Swords casts, attacks, movement) never cancel the shield.
    fn update(&mut self, ctx: &mut Context) { ... }

    /// Draws the shield sprite in front of the knight when active. Runs after
    /// DrawSystem so the shield renders on top of the knight.
    fn draw(&self, _ctx: &Context) { ... }
}
```

# Open Questions
- (none)

# Deferred to Prototype
- Whether the sprite should `flip_x` when the knight faces left (the art is a slightly angled 3/4 shield with a lit edge) is a visual-taste call — try both in-game.
- Whether the sprite wants a slight scale-up, a tint, or a soft pulse while active — taste calls to judge in-game on top of the plain alpha draw.
- `SHIELD_AHEAD` (20px) is a guess: at 64px square the shield overlaps the knight's torso rather than sitting clearly in front. The exact offset (clear the body vs. overlap it) is a visual judgment — tune in-game.
- Whether the wind-up should carry any channeling effect (like Swords' golden glow) or stay a plain swipe pose is a feel call — decide against a throwaway build.
- The duration constants (`STANCE_SECS` 0.5, `SHIELD_SECS` 15.0, `FADE_IN_SECS`/`FADE_OUT_SECS` 0.25) are placeholders to tune in-game; they are all named constants, not baked-in magic numbers.

# Out of Scope
- No gameplay effect: no blocking, no damage, no `AttackRect`, no enemy interaction.
- No new skill slots or icon art — the Shield slot already exists in the bar.
- No shader material — the shield is the plain sprite with an alpha fade; nothing reuses the sword_magic shaders here.
- No changes to `SkillSystem`, `SwordsSkillSystem`, `pico_entity_store`, or `src/util/estore.rs`.
- No handling of a despawned knight mid-shield: the battle test scene has no knight-death path, so the shield assumes a living knight and simply stops drawing when the `PlayerOne` → `Knight` lookup fails.
