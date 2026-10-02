# Description
Split the impact-burst work out of `RamHeadAiSystem`. The ram head system only fires a `RamHeadSplatEvent` when its bite connects; a new scene-local `RamHeadSplatSystem` listens for that event, spawns a burst entity into the store, and owns its whole lifecycle (spawn, age, despawn). The burst is drawn through the normal entity pipeline, so no system draw pass is involved.

# TODO
- [ ] Re-add `RamHeadSplatEvent` to `src/events.rs`
- [ ] Add an `ImpactBurst` data type and `ProceduralEffect::ImpactBurst` variant in `src/entity`
- [ ] Create `src/scene/battle_test/sys_ramhead_splat.rs`
- [ ] Strip burst state/drawing from `RamHeadAiSystem` and fire the event instead
- [ ] Register the new system and module
- [ ] Verify with `cargo build -p heroes-of-ironhold-desktop`

# TODO Explanation
## Re-add `RamHeadSplatEvent` to `src/events.rs`
The event carries only the world-space contact point; the splat system derives all visual randomness itself. Restore the `Vec2` import (`use macroquad::prelude::Vec2;`) alongside the existing `Rect` import.

## Add an `ImpactBurst` data type and `ProceduralEffect::ImpactBurst` variant in `src/entity`
The burst becomes a real store entity by riding the existing `ProceduralDrawable`, so `DrawSystem` already renders it (no change to `sys_draw.rs`). Its z-order comes from `ProceduralDrawable.z_idx`, which `ZSortSystem` does not touch (it only rewrites `Animation`/`StaticImage`), so the pinned value survives every frame.

## Create `src/scene/battle_test/sys_ramhead_splat.rs`
Scene-local (per the systems rule). Subscribes to `RamHeadSplatEvent` in `new()`, enqueues in the handler, and in `update()` drains the queue to `store.add` a `ProceduralDrawable` burst, then ages and removes expired bursts. No `draw()` — rendering is the entity pipeline's job.

The splat is pinned just above the hit entity, not above everything: when spawning, resolve the hit entity's z from the store (the max `z_idx` among its `Animation`/`StaticImage` children) and set the burst's `z_idx = hit_z + SPLAT_Z_OFFSET`, where `SPLAT_Z_OFFSET` is a small constant (e.g. `0.005`, less than `ZSortSystem`'s `0.01` sub-part step, so it sits over the hit entity but under the next entity's rank). Register the system after `ZSortSystem`/`RamHeadAiSystem` so the z value it reads is current-frame.

## Strip burst state/drawing from `RamHeadAiSystem` and fire the event instead
Remove the `bursts: Vec<ImpactBurst>` field, the `spawn_burst` / `burst_alpha` / `burst_points` / `fill_star` helpers, and the `draw()` method. Where the contact point is computed, `self.bus.fire(&RamHeadSplatEvent { position: contact, target: target_id })` replaces `self.bursts.push(...)`.

## Register the new system and module
`mod sys_ramhead_splat;` in `src/scene/battle_test.rs`, `use super::sys_ramhead_splat::RamHeadSplatSystem;` plus `self.agg.add(RamHeadSplatSystem::new(self.store.clone(), self.bus.clone()));` in `src/scene/battle_test/scene.rs`.

## Verify
`cargo build -p heroes-of-ironhold-desktop` from the repo root; fix any warnings/errors.

# Objects
## RamHeadSplatEvent (new)
Event fired by the ram head AI the frame its attack connects; lives in `src/events.rs`.

```rust
#[derive(Clone, Debug)]
pub struct RamHeadSplatEvent {
    pub position: Vec2, // world-space contact point where the burst appears
    pub target: u64,    // id of the entity that was hit; the splat pins its z just above this
}
```

## ImpactBurst (new)
Burst shape data carried by the `ProceduralDrawable` effect; lives in `src/entity/procedural_drawable.rs`.

```rust
#[derive(Clone, Debug)]
pub struct ImpactBurst {
    pub pos: Vec2,               // world-space center of the burst
    pub age: f32,                // seconds since spawn; removed at the burst duration
    pub radii: Vec<f32>,         // alternating outer/inner spike radii, jittered once at spawn
    pub rotation: f32,           // per-burst tilt
    pub action_angles: Vec<f32>, // radiating action-line angles
}
```

## ProceduralEffect (modify)
Adds a variant so `ProceduralDrawable` can represent the burst; lives in `src/entity/procedural_drawable.rs`.

```rust
#[derive(Clone, Debug)]
pub enum ProceduralEffect {
    Consecration(ConsecrationData),
    ImpactBurst(ImpactBurst), // added
}
```

## RamHeadAiSystem (modify)
Drops burst ownership; lives in `src/scene/battle_test/sys_ramhead_ai.rs`.

```rust
pub struct RamHeadAiSystem {
    pub store: Rc<EStore>,
    pub bus: Rc<EventBus>,
    pub states: HashMap<u64, RamBrain>,
    // bursts: Vec<ImpactBurst>, // removed
}
```

## RamHeadSplatSystem (new)
Listens for `RamHeadSplatEvent` and owns the burst entity lifecycle; lives in `src/scene/battle_test/sys_ramhead_splat.rs`.

```rust
pub struct RamHeadSplatSystem {
    store: Rc<EStore>,
    queue: Rc<RefCell<Vec<RamHeadSplatEvent>>>,
    _subs: Rc<SubCollection>,
}
```

# Services
## RamHeadSplatSystem (new)
Lives in `src/scene/battle_test/sys_ramhead_splat.rs`.

```rust
impl RamHeadSplatSystem {
    /// Subscribes to `RamHeadSplatEvent` and stores the subscription for the system's lifetime
    pub fn new(store: Rc<EStore>, bus: Rc<EventBus>) -> Self { ... }
}

impl System for RamHeadSplatSystem {
    /// Drains queued events to spawn bursts into the store, ages them, and removes expired ones
    fn update(&mut self, ctx: &mut Context) { ... }
}
```

## RamHeadAiSystem (modify)
Lives in `src/scene/battle_test/sys_ramhead_ai.rs`.

```rust
impl RamHeadAiSystem {
    // ... existing methods ...

    // fn spawn_burst(pos: Vec2) -> ImpactBurst { ... }   // removed
    // fn burst_alpha(age: f32) -> f32 { ... }            // removed
    // fn burst_points(burst: &ImpactBurst, scale: f32) -> Vec<Vec2> { ... } // removed
    // fn fill_star(center: Vec2, points: &[Vec2], color: Color) { ... }     // removed
}

impl System for RamHeadAiSystem {
    /// Fires `RamHeadSplatEvent { position, target }` at the computed contact point instead of pushing a burst
    fn update(&mut self, ctx: &mut Context) { ... }

    // fn draw(&self, _ctx: &Context) { ... } // removed
}
```

# Out of Scope
- We are not changing how the ram head's attack damage is applied (that stays on `EnemyAttackEvent`).
- We are not making the splat a system-draw pass; it goes through the entity pipeline.
- We are not touching `ZSortSystem`, `DrawSystem`, or `sys_draw.rs`.
