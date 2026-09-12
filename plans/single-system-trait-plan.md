# Description
Consolidate the three phase traits (`Update`, `Draw`, `DrawUi`) into a single `System` trait with default no-op methods, and change `SystemAgg` to hold one list of boxed systems. Systems then override only the phases they need, and registration collapses to a single `agg.add(system)` call. This removes the `#[derive(Clone)]` per-phase-registration footgun: multi-phase systems become a single shared instance, and clone-only `Rc<RefCell>` state simplifies to plain fields (event-handler-held `Rc<RefCell>` queues stay).

# TODO
- [ ] Add `System` trait and remove `Update`/`Draw`/`DrawUi` in `src/systems/sys_aggregate.rs`
- [ ] Rework `SystemAgg` to a single `RefCell<Vec<Box<dyn System>>>`
- [ ] Update `src/systems/mod.rs` re-exports
- [ ] Migrate every system's `impl Update/Draw/DrawUi` blocks into `impl System`
- [ ] Drop `#[derive(Clone)]` and `.clone()` registration for multi-phase systems
- [ ] Update all registration call sites to `agg.add(...)`
- [ ] Verify the workspace builds

# TODO Explanation
## Add `System` trait and remove `Update`/`Draw`/`DrawUi` in `src/systems/sys_aggregate.rs`
Replace the three traits with one `System` trait whose three methods have empty default bodies. A system that does not draw simply does not override `draw`; no explicit no-op bodies are written per system.

## Rework `SystemAgg` to a single `RefCell<Vec<Box<dyn System>>>`
Replace the three phase vectors (`updates`, `draws`, `uis`) with one `systems` vector. The aggregate still runs three sequential passes over that single list — `update` (all systems, `iter_mut()`), then `draw` (all systems, `iter()`), then `draw_ui` (all systems, `iter()`) — never per-system interleaving. Sequential phases share the single instances, so a system's `update` mutations are visible to its own `draw`.

## Update `src/systems/mod.rs` re-exports
Export `System` and `SystemAgg`; drop the `Update`, `Draw`, `DrawUi` exports.

## Migrate every system's `impl Update/Draw/DrawUi` blocks into `impl System`
For each of the 21 systems, merge the existing trait impl blocks into a single `impl System` block, keeping only the methods the system actually implements. Single-phase systems keep one method; multi-phase systems keep two or three. Representative before/after for a three-phase system:

```rust
// before
impl Update for KnightDashSystem {
    fn update(&mut self, ctx: &mut Context) { ... }
}
impl Draw for KnightDashSystem {
    fn draw(&self, ctx: &Context) { ... }
}
impl DrawUi for KnightDashSystem {
    fn draw_ui(&self, ctx: &Context) { ... }
}

// after
impl System for KnightDashSystem {
    fn update(&mut self, ctx: &mut Context) { ... }
    fn draw(&self, ctx: &Context) { ... }
    fn draw_ui(&self, ctx: &Context) { ... }
}
```

## Drop `#[derive(Clone)]` and `.clone()` registration for multi-phase systems
Systems registered in multiple phases today fall into two patterns, both of which collapse to a single registered instance:
- Clone pattern: `RamHeadAiSystem`, `KnightDashSystem`, `EnemyDeathSystem` are `#[derive(Clone)]`, constructed once, and registered per phase via `.clone()` (requiring shared mutable state in `Rc<RefCell<...>>`).
- Separate-construct pattern: `KnightAttackSystem`, `KnightAttackEffectSystem`, `CollisionRectSystem` are constructed *twice* (one instance for update, one for draw), so per-instance state like `prev_frame`/`prev` lives only on the update instance and the draw instance reads only the store.

After consolidation each system is constructed once and `agg.add`-ed once, so its `update` mutations are visible to its own `draw`. Remove the `Clone` derives. For mutable state that exists only to bridge the update/draw clones (e.g. `EnemyDeathSystem.active`), simplify to plain fields; keep `Rc<RefCell<...>>` where an event-subscription handler closure holds it (e.g. `EnemyDeathSystem.queue`, `RamHeadAiSystem.hit_queue`).

## Update all registration call sites to `agg.add(...)`
Every `agg.add_update(...)`, `agg.add_draw(...)`, and `agg.add_ui(...)` call in the scenes becomes a single `agg.add(...)`, in the intended run order. Multi-phase systems registered with `.clone()` are collapsed to one call. The single list's order follows the current **update** registration order (update is the logic-critical pass); the draw and draw_ui passes then reuse that same order, so draw layering must be re-verified against the previous draw order and adjusted if any layer now sits in the wrong place.

## Verify the workspace builds
Run `cargo check --workspace` and fix any fallout.

# Objects
## System (new)
The single system trait; lives in `src/systems/sys_aggregate.rs`.

```rust
pub trait System: Any + 'static {
    /// Runs simulation each frame; default is a no-op.
    fn update(&mut self, _ctx: &mut Context) {}

    /// Draws to the world/screen space each frame; default is a no-op.
    fn draw(&self, _ctx: &Context) {}

    /// Draws screen-space UI each frame; default is a no-op.
    fn draw_ui(&self, _ctx: &Context) {}
}
```

## Update (remove)
Removed from `src/systems/sys_aggregate.rs`; all implementors migrate to `impl System`.

```rust
pub trait Update: Any + 'static {
    fn update(&mut self, ctx: &mut Context);
}
```

## Draw (remove)
Removed from `src/systems/sys_aggregate.rs`; all implementors migrate to `impl System`.

```rust
pub trait Draw: Any + 'static {
    fn draw(&self, ctx: &Context);
}
```

## DrawUi (remove)
Removed from `src/systems/sys_aggregate.rs`; all implementors migrate to `impl System`.

```rust
pub trait DrawUi: Any + 'static {
    fn draw_ui(&self, ctx: &Context);
}
```

## SystemAgg (modify)
Lives in `src/systems/sys_aggregate.rs`; holds one list of boxed systems instead of three.

```rust
pub struct SystemAgg {
    systems: RefCell<Vec<Box<dyn System>>>, // changed: was updates/draws/uis of Box<dyn Update/Draw/DrawUi>
}
```

# Services
## SystemAgg (modify)
Lives in `src/systems/sys_aggregate.rs`.

```rust
impl SystemAgg {
    /// Returns an empty aggregate.
    pub fn new() -> Self { ... }

    /// Registers a system in all three phases (its no-op methods are inert).
    pub fn add<T: System>(&self, system: T) { ... }

    /// Removes the first system of type `T`; returns whether one was removed.
    pub fn remove<T: System>(&self) -> bool { ... }

    /// Removes every registered system.
    pub fn clear(&self) { ... }

    /// Runs `update` on every system, in registration order, with mutable access.
    pub fn update(&self, ctx: &mut Context) { ... }

    /// Runs `draw` on every system, in registration order.
    pub fn draw(&self, ctx: &Context) { ... }

    /// Runs `draw_ui` on every system, in registration order.
    pub fn draw_ui(&self, ctx: &Context) { ... }

    // removed: add_update, add_draw, add_ui, remove_update, remove_draw, remove_ui
}
```

# Open Questions

# Out of Scope
- No changes to the `pico_entity_store` crate or `src/util/estore.rs`.
- No intentional re-ordering of the update pass: the single list follows the current update registration order, and only the draw/draw_ui layering is re-verified against it.
- No renaming of the `SystemAgg` type or the `src/systems` module itself.
- No changes to `rules/systems.md`, `rules/events.md`, or other docs (unless explicitly requested).
