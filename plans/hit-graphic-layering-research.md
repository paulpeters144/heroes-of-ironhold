# Hit Graphic Layering

How the game decides what the ram-head impact burst (the comic "hit graphic") is
drawn over and what is drawn over it. The answer is not a single z-index — the
game mixes two unrelated ordering mechanisms, and the burst uses neither.

## Two ordering mechanisms coexist

1. **System draw order** — systems registered on the scene's `SystemAgg` draw in
   insertion order. This is the ONLY thing that positions the burst.
2. **Per-entity `z_idx` sort** — store entities (`Animation`, `StaticImage`,
   `HealthBar`, `FloatingText`, `ProceduralDrawable`) are collected and sorted by
   `z_idx` inside `DrawSystem`. This never touches the burst.

## The burst has no z-index at all

The burst is drawn directly by `RamHeadAiSystem::draw` (`sys_ramhead_ai.rs:593`)
with plain macroquad calls (`draw_line`, `draw_triangle`). It is stored in a
`Vec<ImpactBurst>` field on the system — it is **not** an entity, is never added
to the store, and therefore never appears in `DrawSystem` or `ZSortSystem`. Its
position is fixed entirely by where `RamHeadAiSystem` sits in the aggregate.

## System draw order = registration order

`SystemAgg` (`src/systems/sys_aggregate.rs`) keeps a `Vec<Box<dyn System>>`;
`add()` pushes to the end and `draw()` iterates in that order. There is no
z-index or layer field — later registration draws later (on top). Systems are
wired in `BattleTestScene::load` (`scene.rs:324-423`).

Draw systems in registration order (world-space `draw` pass):

| Order | System (`scene.rs` line) | Layer |
|-------|--------------------------|-------|
| 1 | MapDrawSystem (325) | map (bottom) |
| 2 | DivineStanceSystem (326) | divine stance FX — **under** entities |
| 3 | DrawSystem (331) | all store entities, z-sorted internally |
| 4 | ConsecrationSystem (332) | consecration ground |
| 5 | BladeBarrageSkillSystem (354) | flying swords |
| 6 | ShieldTossSystem (359) | shield toss |
| 7 | KnightAttackEffectSystem (372) | knight slash/thrust graphics |
| 8 | **RamHeadAiSystem (382)** | **impact burst draws here** |
| 9 | DebugDrawSystem (411) | debug outlines (only when `ctx.debug`) |
| 10 | EnemyDeathSystem (414) | death scorch/soul FX |

So the burst is drawn **over**: the map, divine stance, every character/sprite
and the y-sorted entity layer (including floating damage text and health bars),
consecration, blade-barrage swords, shield toss, and knight attack slashes.

It is drawn **under**: the enemy-death effects and the debug overlays.

### What the entity layer is doing underneath

`DrawSystem::draw` (`sys_draw.rs:29`) gathers every visible `Animation`,
`StaticImage`, `HealthBar`, `FloatingText`, and `ProceduralDrawable` into
`DrawCmd { z_idx, .. }`, sorts by `z_idx` (via each type's `zdx()`), then draws.
So *within* the character layer, higher `z_idx` wins.

`ZSortSystem` (`sys_z_sort.rs`) rewrites every entity's `z_idx` each update:
- groups entities by owner (character), keyed on the character's **feet**
  (`rect.y + rect.h`, ascending = further down = drawn later/on top),
  then x, then entity id;
- sub-parts (`Sword`/`Shield`) get `+SUB_PART_STEP` (0.01) increments so they
  sit slightly above their owner;
- assigns `rank + part_idx * 0.01`.

This is the classic painter's-algorithm y-sort. The burst ignores it completely:
because `RamHeadAiSystem` runs *after* the whole `DrawSystem` pass, the burst is
always on top of every entity regardless of that entity's y position.

`FloatingText` uses a hard-coded `ON_TOP_Z = 1000.0` (`floating_text.rs:14`), so
damage numbers sort above all characters *within* `DrawSystem` — but they still
render before the burst, so the burst is drawn over damage numbers too.

## The UI pass is always last

`Manager::draw` (`src/manager/mod.rs` / `manager.rs:165`) runs:

1. `begin_scene_pass` — camera centered on `ctx.cam_target`.
2. `draw_scene` — `scene.draw(ctx)` → `agg.draw(ctx)`, i.e. everything above
   including the burst.
3. `begin_screen_pass` — `set_default_camera()`.
4. `blit_target` — render target → screen.
5. `draw_ui` — `scene.draw_ui(ctx)` → `agg.draw_ui(ctx)` with its own screen-space
   camera.

Any `draw_ui` system (`HudDrawSystem`, `SkillSystem`) renders after and on top of
the whole world pass, so the HUD/skill bar always covers the burst.

## Consequences

- The burst is never occluded by characters, no matter how tall/overlapping; it
  always paints over the entire entity layer.
- It **is** occluded by enemy-death FX and (in debug mode) the debug outlines,
  because those systems register later.
- To change what the burst draws over/under, move the `RamHeadAiSystem` line in
  `scene.rs` relative to the other draw systems (or move the burst draw call into
  its own system at the desired slot) — there is no z-index to tweak on the burst
  itself.

## Key code locations

- `src/scene/battle_test/sys_ramhead_ai.rs:593` — burst `draw` (no z_idx).
- `src/scene/battle_test/scene.rs:324-423` — system registration order.
- `src/systems/sys_aggregate.rs` — `SystemAgg` insertion-order `update/draw/draw_ui`.
- `src/systems/sys_draw.rs` — z-sorted entity draw pass.
- `src/systems/sys_z_sort.rs` — recomputes entity `z_idx` (y-sort).
- `src/entity/floating_text.rs:14` — `ON_TOP_Z = 1000.0`.
- `src/manager.rs:165-223` — scene pass → blit → UI pass ordering.
