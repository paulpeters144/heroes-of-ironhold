# Description
Add a new system `CollisionRectSystem` in `src/systems/sys_collision_rect.rs` that (1) keeps the knight's `CollisionRect` centered on its body's bounding box (the center is derived from the body `Animation`'s `rect()`), and (2) draws the rects when debug mode is on. The `CollisionRect` component was added earlier but is never moved, so the rects stay pinned at the origin while the knight walks. The current standalone `DebugCollisionRectSystem` (in `sys_debug_collision_rect.rs`) is removed — its draw code folds into `CollisionRectSystem` and is gated by a new `debug` flag read from `Config`.

# Objects

## Config (modify)
Adds a `debug` field to the existing struct in `src/util/config.rs`, defaulted to false. A generic master switch that drives whether debug visuals are drawn; collision rects are the first consumer.

```rust
pub struct Config {
    // ... existing fields ...
    pub debug: bool, // master switch for all debug visuals; collision rects are the first consumer
}
```

# Services

## CollisionRectSystem (new)
A system living in `src/systems/sys_collision_rect.rs`, implementing both `Update` (move rects) and `Draw` (render debug outlines). Position sync is top-down: it iterates every `Knight` and centers that knight's `CollisionRect` child on its body `Animation` child. The traversal follows the store-access rules in `rules/store-access.md`. The `draw_debug` flag is captured from `cfg.debug` at construction, so the system never touches `Assets`/`Config` accessors from inside `update`/`draw`.

```rust
use crate::{Animation, CollisionRect, Context, EStore, Knight};

pub struct CollisionRectSystem {
    store: Rc<EStore>,
    draw_debug: bool, // whether the collision rect outlines are drawn
}

impl CollisionRectSystem {
    /// Constructs the system with the shared entity store and the debug flag read from config.
    pub fn new(store: Rc<EStore>, draw_debug: bool) -> Self { ... }
}

impl Update for CollisionRectSystem {
    /// For each Knight, centers its CollisionRect child on its body Animation child's
    /// bounding box: sets rect.x = center.x - rect.w/2 and rect.y = center.y - rect.h/2,
    /// where `center` is the body Animation's rect() center. rect.w/rect.h are unchanged.
    fn update(&mut self, _ctx: &mut Context) { ... }
}

impl Draw for CollisionRectSystem {
    /// Draws each CollisionRect's outline when `draw_debug` is set; no-op otherwise.
    fn draw(&self, _ctx: &Context) { ... }
}
```

## DebugCollisionRectSystem (remove)
Removed from `src/systems/sys_debug_collision_rect.rs` (file deleted). Its `Draw` impl body moves into `CollisionRectSystem::draw`, gated behind `draw_debug`. Callers: the `pub use` in `src/systems/mod.rs` and the `.add_draw(DebugCollisionRectSystem::new(...))` registration in `src/scene/battle_test/scene.rs` are both dropped; the scene registers `CollisionRectSystem` as both an update and draw system instead.

```rust
pub struct DebugCollisionRectSystem {
    store: Rc<EStore>,
}

impl Draw for DebugCollisionRectSystem {
    fn draw(&self, _ctx: &Context) { ... }
}
```

# TODO
- [ ] Add `debug: bool` to `Config` in `src/util/config.rs` (default `false`)
- [ ] Add `src/systems/sys_collision_rect.rs` with `CollisionRectSystem` implementing `Update` and `Draw`
- [ ] Delete `src/systems/sys_debug_collision_rect.rs` and remove its export from `src/systems/mod.rs`
- [ ] Export `CollisionRectSystem` from `src/systems/mod.rs`
- [ ] In `BattleTestScene`, register `CollisionRectSystem` as an update and a draw system, drop the `DebugCollisionRectSystem` registration, and import `CollisionRectSystem` in `src/scene/battle_test/scene.rs`
- [ ] Verify with `cargo check` / `cargo run -p heroes-of-ironhold-desktop`

# TODO Explanation
## Add `debug: bool` to `Config` in `src/util/config.rs`
Add the `debug` field (type `bool`) to the `Config` struct and initialize it to `false` in `Config::default`. It lives alongside `pixel_snap` etc. as a plain toggle. The scene passes `self.cfg.debug` to `CollisionRectSystem::new`.

## Add `src/systems/sys_collision_rect.rs` with `CollisionRectSystem` implementing `Update` and `Draw`
Create the file with `new(store: Rc<EStore>, draw_debug: bool)` storing both fields. In `update`, the system first gathers every `Knight`'s `EntityRef` via `store.all::<Knight>()` into a `Vec` (gathering first, as `sys_z_sort.rs` does, avoids holding the store's read lock while mutating). For each knight it re-fetches the guard with `get_by_id::<Knight>(id)`, then resolves that knight's body `Animation` child via `store.get_child::<Animation>(&knight)` and its `CollisionRect` child via `store.get_child::<CollisionRect>(&knight)`. It derives the body center from `Animation::rect()` (i.e. `position + dest_size*scale/2`) and writes the rect so it is centered on that point via an in-place `store.update::<CollisionRect, _>(...)` guard that sets `rect.x = center.x - rect.w/2` and `rect.y = center.y - rect.h/2`. The read chains must end in `Copy` values (`Rect` and `EntityRef`) so the read guards are dropped before `store.update` takes the write lock. The rect's `w`/`h` and the component's meaning (vulnerable area) are unchanged. If a knight has no body `Animation` child or no `CollisionRect` child, that knight is skipped silently. In `draw`, iterate `store.all::<CollisionRect>()` and draw each rect's outline with `draw_rectangle_lines` (red `Color::new(1.0, 0.0, 0.0, 1.0)`, 2.0px thickness — preserved from the removed system), but only when `self.draw_debug` is true — otherwise the method returns without drawing.

## Delete `src/systems/sys_debug_collision_rect.rs` and remove its export from `src/systems/mod.rs`
Remove the file and drop the `mod sys_debug_collision_rect;` declaration and `pub use sys_debug_collision_rect::DebugCollisionRectSystem;` line from `src/systems/mod.rs`. `DebugCollisionRectSystem` disappears entirely; its draw logic now lives in `CollisionRectSystem::draw`.

## Export `CollisionRectSystem` from `src/systems/mod.rs`
Add `mod sys_collision_rect;` and `pub use sys_collision_rect::CollisionRectSystem;` to `src/systems/mod.rs`.

## Register `CollisionRectSystem` in `BattleTestScene` and drop `DebugCollisionRectSystem`
In `src/scene/battle_test/scene.rs`, replace `DebugCollisionRectSystem` with `CollisionRectSystem` in the `use crate::systems::{...}` import. In `BattleTestScene::new`, add `agg.add_update(CollisionRectSystem::new(store.clone(), self.cfg.debug))` alongside the other update systems (after `OffsetUpdateSystem`, before `ZSortSystem`). In `load`, replace `.add_draw(DebugCollisionRectSystem::new(self.store.clone()))` with `.add_draw(CollisionRectSystem::new(self.store.clone(), self.cfg.debug))`. Registering the update in `new()` means it runs before `DashSystem` (added later in `load()`), so during a dash the rect trails the body by one frame — accepted, since hit detection is out of scope.

## Verify with `cargo check` / `cargo run -p heroes-of-ironhold-desktop`
Run `cargo check` (or the desktop build) to confirm it compiles. With `debug` defaulted to false the rects are hidden; the `draw_debug` flag can be exercised by temporarily setting `debug` to true in `Config::default`, confirming the red outlines now track the knight's body.

# Open Questions

# Out of Scope
- Collision/hit detection or resolving an attack result
- Resizing or offsetting the collision rect (hurtbox tuning)
- Attaching collision rects to enemies or other entities (the system hardcodes `Knight` as the owner; supporting other owners means generalizing the traversal later)
- Adding other debug visuals gated by `Config::debug`
- Runtime toggling of debug visuals (the flag is read once at construction; no live switch)