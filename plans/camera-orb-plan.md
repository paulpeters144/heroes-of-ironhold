# Description
Add a small "camera orb" — a standalone entity (no parent) rendered as a plain rect that floats a fixed 100 units directly in front of the knight. A dedicated system snaps the orb's position to the knight's center + offset every frame and draws the orb. Camera logic is left untouched; the orb is a static, exact anchor point that the camera will later pan/lerp toward (and will eventually be hidden from the player).

# TODO
- [ ] Add an `Orb` component (position + size) as a root entity
- [ ] Add a `CameraOrbSystem` with `update()` that snaps the orb 100 in front of the knight
- [ ] Add `draw()` to `CameraOrbSystem` that draws the orb as a rect
- [ ] Register the orb entity and system in the scene(s)

# TODO Explanation
## Add an `Orb` component (position + size) as a root entity
- New struct (e.g. `Orb { pos: Vec2, size: f32 }`), `Clone + Send + Sync` so it can live in the `EStore`. Size is 8x8.
- Spawned once as a root entity with no children: `store.add(Orb { .. }, &[])` (no parent).
- Component lives in the knight entity module (or a nearby module) alongside `Knight`/`Sword`/`Shield`, mirroring how those components are defined.

## Add a `CameraOrbSystem` with `update()` that snaps the orb 100 in front of the knight
- New system (e.g. `src/scene/battle_test/sys_orb.rs` or similar) mirroring `sys_camera.rs`.
- `update()` reads the knight via `store.first::<Knight>()` → `get_child::<Animation>()`, computes `orb.pos = animation.rect().center() + vec2(100.0, 0.0)`, and writes it into the `Orb` via `all_mut::<Orb>()`.
- Anchor is the knight's center (`rect().center()`).
- "In front" = +x (the knight faces right), fixed world-space 100 units. Zoom is ignored for now.
- Position is static and exact — instant snap every frame, no lerp/smoothing (the camera will lerp later).

## Add `draw()` to `CameraOrbSystem` that draws the orb as a rect
- `draw()` reads the `Orb` and draws a filled 8x8 rect (`draw_rectangle`) centered on `orb.pos` (i.e. subtract half size), no asset/texture needed. Color: yellow.
- Implements the existing `systems::Draw` trait (and `Update`) so it plugs into the scene/system aggregation.

## Register the orb entity and system in the scene(s)
- Add the `Orb` entity spawn to the battle-test scene's setup only.
- Construct and register `CameraOrbSystem` in the battle-test scene's system aggregation in the correct update/draw order (after knight animation/offset update, before/with other draws). The camera system itself is not modified.

# Open Questions

# Out of Scope
- No camera logic changes (camera still centers on the knight as today).
- No parenting the orb to the knight (it must be a standalone root entity).
- No sprite/texture for the orb (plain rect only).
- No collision, gameplay, or interaction behavior for the orb.
