# Peon and Ram Head Spawn Areas

Research into where peons (hero allies) and ram heads (enemies) spawn in the battle test scene, and how spawning is driven. All coordinates are world units (macroquad Rect: `x, y, w, h`).

## Map context

- Scene: `src/scene/battle_test/` (BattleTestScene), TMX map `assets/tiled/test-scene/test.tmx` (`125 × 25` tiles at `16 × 16` px → world `2000 × 400`).
- `map_w` is passed into both spawner systems as `2000.0` (scene.rs:255 → `RamHeadSpawnerSystem::new(..., map_w)` and `PeonSystem::new(..., map_w)`).
- Both spawn zones are registered as `AreaRect` components (generic debug-visible rects, drawn by `sys_debug_draw.rs:23`), so they show up in debug draws.

## Peon spawn area — left edge of the map

File: `src/scene/battle_test/sys_peon.rs`

- Constants (sys_peon.rs:54-57):
  - `SPAWN_AREA_X = 0.0`, `SPAWN_AREA_Y = 125.0`, `SPAWN_AREA_W = 50.0`, `SPAWN_AREA_H = 200.0`
- Area rect: `(0.0, 125.0, 50.0, 200.0)` → world x `0..50`, y `125..325` (far-left strip of the map, no room off-screen left; the map's left collision object starts at x ≈ 0).
- Registered in `PeonSystem::new()` as an `AreaRect` (sys_peon.rs:174-179).
- `spawn_peon()` (sys_peon.rs:197-236) picks a random point inside the rect: `x = gen_range(0.0, 50.0)`, `y = gen_range(125.0, 325.0)`, and sets the body `Animation` position there.
- Driven by `SpawnPeonSquadEvent { count }` (events.rs:88) fired by the wave director during its `Gap` phase; count comes from `wave.peon_squad` (sys_wave_director.rs:143-148).
- Post-spawn behavior: peons form up in front of the knight (`formation_slot`, sys_peon.rs:267) and walk right toward ram heads; despawn when `x > map_w + 80` (sys_peon.rs:615) or turn back (`Return` state) when more than `200` past the knight (sys_peon.rs:619).

## Ram head spawn area — right edge of the map

File: `src/scene/battle_test/sys_ramhead_spawner.rs`

- Constants (sys_ramhead_spawner.rs:14-16): `SPAWN_AREA_Y = 125.0`, `SPAWN_AREA_W = 50.0`, `SPAWN_AREA_H = 200.0`
- `spawn_area_x = map_w - SPAWN_AREA_W = 2000 - 50 = 1950` (sys_ramhead_spawner.rs:39)
- Area rect: `(1950.0, 125.0, 50.0, 200.0)` → world x `1950..2000`, y `125..325` (far-right strip of the map, mirroring the peon zone vertically).
- Registered in `RamHeadSpawnerSystem::new()` as an `AreaRect` (sys_ramhead_spawner.rs:40-45).
- `spawn_ram_head()` (sys_ramhead_spawner.rs:57-96) picks a random point inside the rect: `x = spawn_area_x + gen_range(0.0, 50.0)`, `y = SPAWN_AREA_Y + gen_range(0.0, 200.0)`.
- Driven by `SpawnRamHeadEvent { count }` (events.rs:81) fired by the wave director during its `Burst` phase; each burst ticks one spawn per `spawn_interval` (sys_wave_director.rs:126-131).
- Post-spawn behavior: ram heads stalk/charge toward the knight or nearest peon (`nearest_target`, sys_ramhead_ai.rs:183), so they naturally flow left from the right edge. Y clamped to `40..360`.

## Wave director driving the spawns

File: `src/scene/battle_test/sys_wave_director.rs`

- The director is the only subscriber-side producer: it fires `SpawnRamHeadEvent` during `Burst` and `SpawnPeonSquadEvent` during `Gap`. It does not spawn anything itself — the spawner systems subscribe and own the actual spawning.
- Waves (scene.rs:278-317), gate `at_x` thresholds vs the knight's x:
  - Wave 0: `at_x 0`, 5 ram heads @ 0.2s, gap 3.0s, peon squad 2
  - Wave 1: `at_x 600`, 4 @ 0.4s, gap 4.0s, peon squad 3
  - Wave 2: `at_x 1000`, 6 @ 0.3s, gap 4.0s, peon squad 3
  - Wave 3: `at_x 1400`, 8 @ 0.25s, gap 5.0s, peon squad 4
  - Wave 4: `at_x 1700`, 10 @ 0.2s, gap 0.0s, peon squad 0
  - Global cap: `max_alive: 6` live ram heads (sys_wave_director.rs:111).
- Rest nodes (safe zones where ram head spawning pauses while the knight is inside): `RestNode` rects `(820, 40, 160, 320)` and `(1260, 40, 160, 320)` (scene.rs:319-330), checked via `in_rest_node` (sys_wave_director.rs:81). These sit between wave gates 600/1000 and 1000/1400.

## Summary

| | Peons | Ram heads |
|---|---|---|
| Spawn rect | `(0, 125, 50, 200)` | `(1950, 125, 50, 200)` |
| Map edge | Left (x 0–50) | Right (x 1950–2000) |
| Vertical band | y 125–325 | y 125–325 |
| System | `PeonSystem` (sys_peon.rs) | `RamHeadSpawnerSystem` (sys_ramhead_spawner.rs) |
| Event | `SpawnPeonSquadEvent` (Gap phase) | `SpawnRamHeadEvent` (Burst phase) |
| Producer | `WaveDirectorSystem` | `WaveDirectorSystem` |
| AreaRect debug marker | yes | yes |

Both zones are 50-wide × 200-tall vertical strips, mirror images on the far left/right map edges, both centered vertically on y 125–325, with ram heads coming from the right and peons rallied to the knight on the left.