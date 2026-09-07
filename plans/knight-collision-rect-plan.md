# Description
Add a generic collision rect component, used by all heroes and enemies. It represents the area where a thing can take damage (a hurtbox), not where a thing deals damage. For now it is attached only to the knight as a child of the `Knight` entity. A debug draw system renders the rects so they are visible. No hit-detection logic is implemented yet — only the component, its wiring into the knight spawn, and the debug draw.

# Objects

## CollisionRect
A generic component (child of `Knight`), reusable by heroes and enemies. Represents the entity's vulnerable area (where it takes damage). Wraps a single macroquad `Rect`. Lives in `src/entity/collision_rect.rs`.

```rust
#[derive(Clone, Debug)]
pub struct CollisionRect {
    pub rect: Rect, // the macroquad rect used for overlap tests (x/y/w/h + overlaps())
}
```

## KnightParts (changed)
Adds one field to the existing struct in `src/entity/factory_hero.rs`.

```rust
pub struct KnightParts {
    // ... existing fields ...
    pub collision_rect: CollisionRect, // the rect attached as a child of the knight
}
```

# Services

## DebugCollisionRectSystem
A draw system (implements the `Draw` trait), added to the scene's `SystemAgg`. Lives in `src/systems/sys_debug_collision_rect.rs`.

```rust
impl DebugCollisionRectSystem {
    /// Constructs the system with the shared entity store.
    pub fn new(store: Rc<EStore>) -> Self { ... }
}

impl Draw for DebugCollisionRectSystem {
    /// Draws a visible outline for every CollisionRect component in the store.
    fn draw(&self, ctx: &Context) { ... }
}
```

# TODO
- [ ] Define the `CollisionRect` component in a new `src/entity/collision_rect.rs`
- [ ] Register and export `CollisionRect` in `src/entity/mod.rs` and `src/lib.rs`
- [ ] Add `collision_rect` to `KnightParts` and construct it in `HeroFactory::create_knight`
- [ ] Attach the rect as a child of `Knight` in `BattleTestScene::spawn_player`
- [ ] Add `sys_debug_collision_rect.rs` (in `src/systems/`) with `DebugCollisionRectSystem` and register it in the scene

# TODO Explanation
## Define the `CollisionRect` component in a new `src/entity/collision_rect.rs`
Add a top-level module (analogous to `animation.rs`/`static_image.rs`) with the `CollisionRect` struct holding a single `rect: Rect` field (macroquad `Rect`) and `#[derive(Clone, Debug)]`.

## Register and export `CollisionRect` in `src/entity/mod.rs` and `src/lib.rs`
Add `pub mod collision_rect;` to `entity/mod.rs` and `pub use entity::collision_rect::CollisionRect;` to `lib.rs`.

## Add `collision_rect` to `KnightParts` and construct it in `HeroFactory::create_knight`
Add the `collision_rect: CollisionRect` field to `KnightParts` and initialize it in `create_knight` (analogous to `shield_image`/`sword_animation`).

## Attach the rect as a child of `Knight` in `BattleTestScene::spawn_player`
Include `parts.collision_rect.into_child()` in the `Knight` child list alongside `parts.body`, `parts.shield`, `parts.sword`, etc.

## Add `sys_debug_collision_rect.rs` with `DebugCollisionRectSystem` and register it in the scene
Add the system in `src/systems/sys_debug_collision_rect.rs` (following the `Draw` trait pattern in `sys_draw.rs`) that iterates `store.all::<CollisionRect>()` and draws each rect's outline. Export it from `src/systems/mod.rs` and register it in `BattleTestScene` via `agg.add_draw(DebugCollisionRectSystem::new(store.clone()))` after the normal `DrawSystem`.

# Open Questions

# Out of Scope
- Actual attack hit-detection / overlap logic
- Attaching `CollisionRect` to other entities (enemies, shields, swords) for now
- Applying damage or resolving an attack result
