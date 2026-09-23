# Description

Replace the fixed spawn zones (peons at the map's left edge, ram heads at the map's right edge) with a sliding window: each time the hero reaches a wave gate, the ram head spawn zone is placed ahead of the gate and the peon spawn zone is placed behind it. The camera's left clamp follows the peon spawn zone so the player never sees peons pop into existence.

Resolved decisions: the ram zone sits 400px ahead of the gate (~80px past the right edge of the 640-wide view) and the peon zone sits 80px behind it; only the left (peon) side gets a camera clamp — the ram zone hides pop-in by placement, not clamping; the knight's own movement is never blocked; the existing wave gates stay as they are.

# TODO

- [ ] Add `PeonSpawnZone` / `RamHeadSpawnZone` components and export them from the entity barrel
- [ ] WaveDirectorSystem: compute and write both spawn zones when a wave triggers (sliding window)
- [ ] RamHeadSpawnerSystem: spawn from `RamHeadSpawnZone` in the store instead of the fixed right edge
- [ ] PeonSystem: spawn from `PeonSpawnZone` in the store instead of the fixed left edge
- [ ] CameraSystem: clamp the left edge of the view against the peon spawn zone
- [ ] DebugDrawSystem: draw the two spawn zones; spawners stop registering `AreaRect` debug markers
- [ ] Scene wiring: update system constructor calls in `scene.rs`
- [ ] Verify: build, run unit tests (`clamp_axis`, `spawn_zone_rects`), play-test the wave flow

# TODO Explanation

## Add spawn zone components

Create `src/entity/spawn_zone.rs` holding two single-field components, `PeonSpawnZone` and `RamHeadSpawnZone`, each wrapping a `macroquad::prelude::Rect`. Register them in the `src/entity.rs` barrel (`mod spawn_zone;` + `pub use spawn_zone::{PeonSpawnZone, RamHeadSpawnZone};`) so systems import them via `crate::entity::{...}`. These are pure store data (like `RestNode`): the wave director writes them, the spawners and camera read them. No events are needed — spawn zones are world state, not one-shot notifications.

## WaveDirectorSystem: write the sliding zones

In `src/scene/battle_test/sys_wave_director.rs`, when a wave triggers (the `Waiting` → `Burst` transition, i.e. the knight has reached `wave.at_x`), compute both zone rects relative to the gate x and write them into the store:

- Ram head zone: `x = (gate_x + RAM_ZONE_AHEAD).min(map_w - SPAWN_ZONE_W)` — ahead of the hero. `RAM_ZONE_AHEAD = 400.0`: the virtual view is 640 wide (`Config::v_width`), so at the trigger frame the camera shows up to ~`gate + 320`; 400 puts the zone ~80px past the right edge — no visible pop-in, and ram heads still engage quickly since they stalk/charge toward the knight.
- Peon zone: `x = (gate_x - PEON_ZONE_BEHIND - SPAWN_ZONE_W).max(0.0)` — behind the hero. `PEON_ZONE_BEHIND = 80.0`: peons appear 80–130px behind the gate, hidden by the camera clamp, then walk forward into their formation slots in front of the knight like reinforcements catching up.

The zone height/width constants move here from the two spawner files (`SPAWN_ZONE_Y = 125.0`, `SPAWN_ZONE_W = 50.0`, `SPAWN_ZONE_H = 200.0`, unchanged values). `WaveDirectorSystem::new` gains a `map_w: f32` parameter (for the right-edge clamp) and writes the initial zones for `waves[0].at_x` so the zones exist before the first update. Writing means upserting the single `PeonSpawnZone` / `RamHeadSpawnZone` components in the store (create on first write via `store.add`, `store.update` through the guard's `entity_ref()` afterwards). Rest of the director (phases, rest nodes, `max_alive`, events) is unchanged.

Concrete zones for the current schedule (`map_w = 2000`), which double as the play-test expectations:

| Gate `at_x` | Peon zone (x) | Ram zone (x) | Notes |
|---|---|---|---|
| 0 | `0..50` (clamped at 0) | `400..450` | initial zones, written in `new()` |
| 600 | `470..520` | `1000..1050` | |
| 1000 | `870..920` | `1400..1450` | |
| 1400 | `1270..1320` | `1800..1850` | |
| 1700 | `1570..1620` | `1950..2000` | ram zone pinned to the map edge by the `min(map_w - SPAWN_ZONE_W)` clamp |

Both zones are written on every wave trigger, including the last one whose `peon_squad` is 0 — the zone exists but spawns nothing. Zones only affect spawning; peons and ram heads already alive when a zone moves are completely unaffected.

Design notes:

- The rect math lives in a pure free function `spawn_zone_rects(gate_x, map_w)` (same style as the testable free functions in `sys_camera.rs`) so the clamp cases above are unit-testable without a store.
- Registration order in the scene aggregate matters: the director writes zones and fires `SpawnRamHeadEvent` during its update, and the spawners read the zone when they drain their queues later in the same frame. `WaveDirectorSystem` is already added before `RamHeadSpawnerSystem` and `PeonSystem` in `scene.rs` — keep that order.
- Import fallout: `PeonSpawnZone` / `RamHeadSpawnZone` join the `crate::entity::{...}` list in `sys_wave_director.rs`, and `Rect` joins the `macroquad::prelude` import (currently only `Vec2`).

## RamHeadSpawnerSystem: read zone from store

In `src/scene/battle_test/sys_ramhead_spawner.rs`: drop the `map_w` constructor parameter and the `spawn_area_x` field, remove the `AreaRect` registration in `new()`, and change `spawn_ram_head()` to read the current `RamHeadSpawnZone` from the store (`store.first::<RamHeadSpawnZone>()`) and pick a random point inside its rect (`gen_range` over the rect's x/y and w/h, same pattern as today). If no zone exists yet, skip the spawn. Clean up the imports: `AreaRect` and `Rect` leave the `use` lists, `RamHeadSpawnZone` joins the `crate::entity` import.

## PeonSystem: read zone from store

In `src/scene/battle_test/sys_peon.rs`: remove the `SPAWN_AREA_X` constant and the `AreaRect` registration in `new()`, and change `spawn_peon()` to read the current `PeonSpawnZone` from the store (`store.first::<PeonSpawnZone>()`) and pick a random point inside its rect. `PeonSystem::new` keeps its `map_w` parameter — it is still used for the off-right-edge despawn check. The `SPAWN_AREA_Y/W/H` constants move to the wave director (see above). `AreaRect` leaves the `crate::entity` import list and `PeonSpawnZone` joins it; `Rect` stays (still used by `attack_rect`). Everything downstream of spawning — formation slots anchored to the knight, chase/attack, the `MAX_PAST_KNIGHT` return behavior, the `map_w + DESPAWN_MARGIN` cleanup — is untouched, and peons already alive when the zone moves keep walking their current orders.

## CameraSystem: clamp against the peon zone

In `src/scene/battle_test/sys_camera.rs`: generalize `clamp_axis` to take explicit `min`/`max` bounds, and demote it from a method to a free function alongside `spring_toward` / `step_camera` — it never used `self`, and the free-function style is what the file's existing unit tests already exercise. For the x axis, read `store.first::<PeonSpawnZone>()` each frame; the camera center's minimum becomes `zone.rect.x + zone.rect.w + view_w * 0.5` (so the view's left edge never enters the peon spawn rect), bounded below by the usual `view_w * 0.5` map edge when no zone exists. The maximum stays `map_w - view_w * 0.5`. The y axis keeps the current map clamp via the same free function.

**Edge case — clamp inversion at the last gates.** With `view_w = 640` the peon-zone minimum eventually overtakes the map maximum, and Rust's `f32::clamp` panics when `min > max`. `clamp_axis` must therefore guard internally (`min.min(max)`), which pins the camera to the map's right edge once the zone minimum would exceed it:

| Gate `at_x` | Peon zone right edge | Camera min center (`+ 320`) | Map max center | Effective |
|---|---|---|---|---|
| 0 | 50 | 370 | 1680 | 370 |
| 600 | 520 | 840 | 1680 | 840 |
| 1000 | 920 | 1240 | 1680 | 1240 |
| 1400 | 1320 | 1640 | 1680 | 1640 |
| 1700 | 1620 | 1940 | 1680 | 1680 (pinned) |

At gate 1700 the view necessarily shows the peon zone (`1570..1620`), which is acceptable because the final wave's `peon_squad` is 0 — nothing ever spawns there. A side effect of the sliding clamp: at late gates the minimum sits close to the map maximum, so the knight rides the left third of the screen. That is the intended trade of `PEON_ZONE_BEHIND = 80.0` and stays tunable via that one constant.

The clamp only ever raises the minimum, so when zones jump forward at a gate the target simply moves right and the existing spring (`step_camera`) smooths the transition — no snap, since `snap_next` is only true on the first frame.

Only the left (peon) side is clamped — there is deliberately no right-side clamp against the ram zone. The ram zone is placed past the view edge when the wave triggers and drains immediately as ram heads stalk/charge toward the knight, so a clamp there would only fight the follow behavior the game needs. The knight's own movement is likewise never blocked: zones are inert between waves, so backtracking into one is harmless and an invisible wall would feel worse than the rare off-screen pop-in it prevents.

Tests: the existing `#[cfg(test)]` module only covers `spring_toward` and `step_camera` — it never touches `clamp_axis` — so it keeps passing unchanged. Add new unit tests for `clamp_axis`: clamps to `min` and `max` in the normal case, passes values inside the range through, and returns `max` (not a panic) when `min > max`.

## DebugDrawSystem: draw the zones

In `src/systems/sys_debug_draw.rs`: draw `PeonSpawnZone` and `RamHeadSpawnZone` rects alongside the existing `AreaRect` outlines, iterating `store.all::<T>()` the same way the `AreaRect` loop does. Pick colors not already taken in this draw pass (GREEN for `AreaRect`, YELLOW for `AttackRect`, BLUE/RED for collision): `SKYBLUE` for the peon zone and `ORANGE` for the ram zone. This replaces the spawn-zone `AreaRect` markers the spawners used to register, keeping a single source of truth. `AreaRect` drawing stays for Consecration / Divine Stance.

## Scene wiring

In `src/scene/battle_test/scene.rs`: pass `map_w` into `WaveDirectorSystem::new` and drop `map_w` from `RamHeadSpawnerSystem::new`. `PeonSystem::new` keeps its current arguments. No changes to the wave schedule or rest nodes — the existing `at_x` gates (0 / 600 / 1000 / 1400 / 1700) already encode the "move forward N" rhythm and were tuned for the map, so they stay as they are.

## Verify

`cargo build`, then `cargo test -p heroes-of-ironhold-core` — the existing `sys_camera` spring/step tests keep passing untouched, and the new `clamp_axis` (normal, min > max) and `spawn_zone_rects` (gate-0 left clamp, last-gate right clamp) tests pass. Then `cargo run -p heroes-of-ironhold-desktop` from the repo root and confirm against the zone table above: wave 0 ram heads spawn at `x 400..450` just ahead of the hero (not at the far map edge), peons spawn just behind the gate and walk into formation, both zones advance per gate, the camera never reveals the peon zone when walking back left, and at the final gate the camera pins to the map's right edge without panicking.

# Objects

## PeonSpawnZone (new)

Store marker holding the current peon spawn rect. Lives in `src/entity/spawn_zone.rs`, re-exported from `src/entity.rs`.

```rust
use macroquad::prelude::Rect;

/// The sliding window where peons spawn, behind the current wave gate.
/// Written by the wave director, read by the peon system and the camera clamp.
#[derive(Clone, Debug)]
pub struct PeonSpawnZone {
    pub rect: Rect, // world-space spawn area for peons
}
```

## RamHeadSpawnZone (new)

Store marker holding the current ram head spawn rect. Lives in `src/entity/spawn_zone.rs`, re-exported from `src/entity.rs`.

```rust
use macroquad::prelude::Rect;

/// The sliding window where ram heads spawn, ahead of the current wave gate.
/// Written by the wave director, read by the ram head spawner.
#[derive(Clone, Debug)]
pub struct RamHeadSpawnZone {
    pub rect: Rect, // world-space spawn area for ram heads
}
```

## WaveDirectorSystem (modify)

In `src/scene/battle_test/sys_wave_director.rs`: new zone constants and a `map_w` field for the right-edge clamp; all existing fields unchanged.

```rust
// Zone geometry, moved here from sys_peon.rs / sys_ramhead_spawner.rs.
const SPAWN_ZONE_Y: f32 = 125.0; // top of both spawn zones
const SPAWN_ZONE_W: f32 = 50.0; // width of both spawn zones
const SPAWN_ZONE_H: f32 = 200.0; // height of both spawn zones
const RAM_ZONE_AHEAD: f32 = 400.0; // ram zone distance ahead of the wave gate (~80px past the 640-wide view's right edge)
const PEON_ZONE_BEHIND: f32 = 80.0; // gap between gate and peon zone's right edge

pub struct WaveDirectorSystem {
    // ... existing fields (store, bus, schedule, phase, wave_index, burst_remaining,
    // burst_timer, gap_timer, peon_rallied, done) ...
    map_w: f32, // added: world width, used to clamp the ram zone at the map edge
}
```

## RamHeadSpawnerSystem (modify)

In `src/scene/battle_test/sys_ramhead_spawner.rs`: the fixed right-edge field goes away; the zone now lives in the store.

```rust
pub struct RamHeadSpawnerSystem {
    store: Rc<EStore>,
    body: Texture2D,
    impact: Texture2D,
    spawn_area_x: f32, // removed: zone is read from RamHeadSpawnZone in the store
    spawn_queue: Rc<RefCell<VecDeque<SpawnRamHeadEvent>>>,
    _subs: Rc<SubCollection>,
}
```

## PeonSystem (modify)

In `src/scene/battle_test/sys_peon.rs`: the fixed left-edge constant goes away; struct fields unchanged (`map_w` stays for the off-right-edge despawn check).

```rust
const SPAWN_AREA_X: f32 = 0.0; // removed: zone x comes from PeonSpawnZone in the store
const SPAWN_AREA_Y: f32 = 125.0; // removed: moved to sys_wave_director.rs
const SPAWN_AREA_W: f32 = 50.0; // removed: moved to sys_wave_director.rs
const SPAWN_AREA_H: f32 = 200.0; // removed: moved to sys_wave_director.rs
```

# Services

## WaveDirectorSystem (modify)

In `src/scene/battle_test/sys_wave_director.rs`.

```rust
impl WaveDirectorSystem {
    /// Creates the director and writes the initial spawn zones for the first
    /// wave gate — changed: was (store, bus, schedule)
    pub fn new(store: Rc<EStore>, bus: Rc<EventBus>, schedule: WaveSchedule, map_w: f32) -> Self { ... }

    /// Upserts the single PeonSpawnZone and RamHeadSpawnZone components in the
    /// store from the gate x: ram zone ahead of the gate, peon zone behind it,
    /// each clamped to the map. Called on every Waiting -> Burst transition
    /// and once from new() for the first gate.
    fn set_spawn_zones(&self, gate_x: f32) { ... }

    // ... existing private helpers (knight_pos, live_ram_heads, in_rest_node) unchanged ...
}

/// Returns (peon_zone, ram_zone) for a gate: peon zone PEON_ZONE_BEHIND behind
/// the gate clamped at x 0, ram zone RAM_ZONE_AHEAD ahead clamped at the map's
/// right edge. Pure free function so the clamp cases are unit-testable.
fn spawn_zone_rects(gate_x: f32, map_w: f32) -> (Rect, Rect) { ... }

impl System for WaveDirectorSystem {
    /// Same phases as today; Waiting -> Burst additionally calls set_spawn_zones(wave.at_x).
    fn update(&mut self, ctx: &mut Context) { ... }
}
```

## RamHeadSpawnerSystem (modify)

In `src/scene/battle_test/sys_ramhead_spawner.rs`.

```rust
impl RamHeadSpawnerSystem {
    /// Creates the spawner. No longer registers an AreaRect debug marker or
    /// computes a fixed right-edge zone — changed: map_w parameter removed
    pub fn new(store: Rc<EStore>, bus: Rc<EventBus>, assets: &Assets) -> Self { ... }

    /// Spawns one ram head at a random point inside the current RamHeadSpawnZone
    /// from the store; no-op if the zone does not exist yet
    fn spawn_ram_head(&self) { ... }
}

impl System for RamHeadSpawnerSystem {
    /// Unchanged: drains the spawn queue, spawning event.count ram heads
    fn update(&mut self, _ctx: &mut Context) { ... }
}
```

## PeonSystem (modify)

In `src/scene/battle_test/sys_peon.rs`.

```rust
impl PeonSystem {
    /// Unchanged signature (map_w stays for the despawn check); no longer
    /// registers an AreaRect debug marker
    pub fn new(store: Rc<EStore>, bus: Rc<EventBus>, map_w: f32, assets: &Assets) -> Self { ... }

    /// Spawns one peon at a random point inside the current PeonSpawnZone
    /// from the store; no-op if the zone does not exist yet
    fn spawn_peon(&self) { ... }

    // ... all other methods unchanged ...
}
```

## CameraSystem (modify)

In `src/scene/battle_test/sys_camera.rs`.

```rust
impl CameraSystem {
    /// Unchanged
    pub fn new(store: Rc<EStore>, view_w: f32, view_h: f32, map_w: f32, map_h: f32) -> Self { ... }
}

/// Clamps a camera-center value into [min, max], pinning to `max` when
/// min > max instead of panicking — changed: was the private method
/// clamp_axis(&self, value, map_size, view_size) deriving bounds from the map;
/// now a free function next to spring_toward/step_camera so it is unit-testable
fn clamp_axis(value: f32, min: f32, max: f32) -> f32 { ... }

impl System for CameraSystem {
    /// Same spring follow, but the x clamp's minimum is
    /// peon_zone.rect.x + peon_zone.rect.w + view_w/2 when a PeonSpawnZone
    /// exists, so the view never enters the peon spawn area; y unchanged.
    fn update(&mut self, ctx: &mut Context) { ... }
}
```

## DebugDrawSystem (modify)

In `src/systems/sys_debug_draw.rs`.

```rust
impl System for DebugDrawSystem {
    /// Additionally outlines every PeonSpawnZone and RamHeadSpawnZone rect
    /// (distinct colors) when ctx.debug is on; AreaRect drawing unchanged
    fn draw(&self, ctx: &Context) { ... }
}
```

# Out of Scope

- No changes to wave gates, counts, spawn intervals, `max_alive`, or rest node placement — the existing `at_x` spacing stays
- No changes to peon/ram head AI, formation, or combat behavior after spawning
- No changes to `AreaRect` usage by Consecration / Divine Stance
- No right-side camera clamp against the ram zone (placement past the view edge hides pop-in instead)
- No player-movement blocking — the knight can still walk anywhere; only the camera is clamped
- No menu or asset-preview scene changes; this touches only the battle test scene
