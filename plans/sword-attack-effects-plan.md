# Description
Add transient attack-effect graphics to the knight in the asset preview scene. While the knight thrusts, draw a ">>>" chevron graphic in front of the sword tip (to the right, since the knight faces right). While the knight swipes, draw an arc shaped like a backwards uppercase "C" ("Ɔ", opening left) in front of the knight. These are drawn with macroquad primitives (lines), not image assets.

# TODO
- [ ] Expose the attack frame constants (`THRUST_FRAME`, `SWIPE_FRAME`) so other systems can read them
- [ ] Create `src/scene/asset_preview/sys_attack_effects.rs` — a `Draw` system that locates the knight and reads its current frame/position
- [ ] Draw the ">>>" thrust effect in front of the sword when the knight is on the thrust frame
- [ ] Draw the backwards-"C" swipe effect in front of the knight when on the swipe frame
- [ ] Register the new system in `AssetPreviewScene::new()` and declare it in `mod.rs`
- [ ] Build and visually verify both effects look correct when the knight faces right

# TODO Explanation

## Expose the attack frame constants
`THRUST_FRAME = 1` and `SWIPE_FRAME = 2` are currently private `const`s at the top of `src/scene/asset_preview/sys_knight_controls.rs`. The effect system (a sibling module) needs to know these values to detect which attack is active. Change them to `pub(crate) const` (or `pub(super) const`) so `sys_attack_effects.rs` can reference them. Alternatively move both into `model.rs`, but the minimal change is just widening visibility.

## Create `sys_attack_effects.rs`
A new `Draw` system (implements `crate::systems::Draw`) with a `new(store: &'static EStore)` constructor, mirroring the other asset-preview systems. Its `draw(&self, _ctx: &Context)`:

1. Locate the "held" knight the same way `OffsetUpdateSystem::update` does: iterate `store.all::<Knight>()`, pick the one that has an `EntityOffsets` child, and read its child `Animation` to get `position` and `current_frame`.
2. `match animation.current_frame`:
   - `THRUST_FRAME` => draw the chevrons (see below)
   - `SWIPE_FRAME` => draw the arc (see below)
   - anything else => draw nothing (effect only appears during the Strike phase, since the attack frame is only set then)

The knight body `position` is the top-left of its 64x64 frame, so all effect coordinates are computed as `body + offset`.

## Draw the ">>>" thrust effect
The sword during thrust sits at `body + (60, 8)` with a 32x32 size, so its tip is around `body + (92, 24)` (`src/entity/entity_offsets.rs`, frame 1). The chevrons go directly in front of the knight image on its right side. Each chevron should be half the height of the knight frame — `FRAME_SIZE` is 64, so each ">" is 32px tall. Draw three ">" chevrons using `draw_line` (each ">" is two segments meeting at a point), spaced horizontally, vertically centered on the body (i.e. around `body.y + 32`), starting just past the right edge of the body (`body.x + 64`). A small helper like `draw_chevron(tip: Vec2, size: f32, color: Color)` keeps it tidy. Use solid white (`WHITE`, alpha 1.0) for now; exact spacing will be tuned visually.

## Draw the backwards-"C" swipe effect
Macroquad has no `draw_arc`, so build the arc by sampling points along a partial circle and connecting them with `draw_line` segments. The arc should open to the left (concave side facing the player) and bulge to the right into the enemies — a backwards "C" ("Ɔ"). Concretely: pick a center in front of the knight (roughly `body + (48, 32)`), a radius (~24), and sweep an angle range around the +x axis (e.g. -70°..+70°) to form a right-facing, left-opening arc. A helper like `draw_arc(center, radius, start_deg, end_deg, thickness, color)` handles the sampling loop. Use solid white (`WHITE`, alpha 1.0) for now. This is why the body sprite already hides the separate sword/shield on frame 2 (`sword_visible: false`), so the arc won't overlap a duplicate sword.

## Register the new system
In `AssetPreviewScene::new()` (`src/scene/asset_preview/scene.rs`), add `agg.add_draw(AttackEffectSystem::new(store));` **after** `agg.add_draw(DrawSystem::new(store));`. Draw systems run in registration order (`SystemAgg::draw` iterates in insertion order), so registering it after `DrawSystem` makes the effect render on top of the sword/body. Add `mod sys_attack_effects;` to `src/scene/asset_preview/mod.rs`.

## Build and visually verify
`cargo run -p heroes-of-ironhold-desktop` from the repo root. Press Jump to focus the knight, press Attack (X) to thrust, then attack again to swipe. Confirm the ">>>" appears in front of the sword tip on thrust and the backwards-"C" arc appears in front on swipe, both oriented correctly for a right-facing knight. Iterate on colors/positions until they read well.

# Out of Scope
- No new image/asset files — effects are drawn with macroquad primitives only.
- No animation or fade over the effect's lifetime; it is a static graphic shown only on the strike frame (no linger through recovery).
- No hit detection / gameplay damage; these are purely visual.
- No flipping support for a left-facing knight (the knight always faces right for now).
