# Description
Redo the recent "Player owns the hero" work from scratch. The goal is narrowly to introduce a `Player` entity that owns the hero (currently the `Knight`) as its child in the entity store. The previous attempt was out of scope: it added `Player::find` / `Player::class_of` helper methods that duplicated what the store already offers, and moved knight assembly into `HeroFactory`. Those helpers and factory store-methods are removed; scenes and systems query the store directly and fluently — e.g. `store.all::<Player>().find(|p| p.index == idx).and_then(|p| store.get_child::<Knight>(&p))` — expressing ownership by navigating `Player → Knight` with the existing store API. No changes to `EStore` or `pico-entity-store`.

# Objects

## Player (new)
- `#[derive(Clone, Copy, Debug)] pub struct Player { index: u8 }` — the root entity that owns the hero. `index` identifies the player (0 for the first player), reserved for future multi-player indexing. Ownership lives in the store's parent/child hierarchy.

## Hierarchy (established at spawn time, unchanged at runtime)
```
Player
└── Knight
    ├── Animation (body)
    ├── Shield
    │   └── StaticImage (shield image)
    ├── Sword
    │   ├── Animation (sword frames)
    │   ├── Effect (thrust)
    │   └── Effect (slash)
    ├── Facing
    ├── HeroStats
    └── Dash
```

# Services

## PlayerFactory (new, in `src/entity/player/factory_player.rs`)
- `spawn(index: u8) -> Player` — pure constructor; returns a `Player { index }` value. No side effects and no store access.

## HeroFactory (in `src/entity/factory_hero.rs`)
- `create_knight(cfg: KnightCfg) -> KnightParts` — unchanged; builds the raw component values for a knight.

## Removed services
- `Player::find(store: &EStore, index: u8) -> Option<Ref<Player>>` — removed; callers use `store.all::<Player>().find(|p| p.index == index)` directly.
- `Player::class_of<T>(store: &EStore, index: u8) -> Option<Ref<T>>` — removed; callers chain `store.get_child::<T>(&player)` directly.
- `HeroFactory::spawn_knight(store, cfg) -> EntityRef`, `spawn_effects(store, knight)`, `knight_sword(store, knight) -> Option<Ref<Sword>>` — removed; the scene builds the knight subtree with the entity store directly.

# TODO
- [ ] Add the `Player` component and trim `src/entity/player/mod.rs`
- [ ] Add `PlayerFactory::spawn` in `src/entity/player/factory_player.rs`
- [ ] De-helper `factory_hero.rs` (remove `spawn_knight`/`spawn_effects`/`knight_sword`)
- [ ] Update crate exports (`lib.rs`, `entity/mod.rs`, `entity/player/mod.rs`)
- [ ] Rework `asset_preview` scene and systems to query the store fluently
- [ ] Rework `battle_test` scene and systems to query the store fluently
- [ ] Build and run to verify

# TODO Explanation

## Add the `Player` component and trim `src/entity/player/mod.rs`
Create `src/entity/player/components.rs` with the `Player { index: u8 }` component and make `src/entity/player/mod.rs` a plain module that only re-exports it (matching the `enemy`/`dash`/`skills` module shape). Remove the `impl Player { find, class_of }` block and its `EStore`/`Ref` imports from `mod.rs`.

## Add `PlayerFactory::spawn` in `src/entity/player/factory_player.rs`
Replace the current `src/entity/player/factory.rs` with `factory_player.rs`. `PlayerFactory::spawn(index)` is a pure constructor that returns `Player { index }` — no store, no side effects.

## De-helper `factory_hero.rs` (remove `spawn_knight`/`spawn_effects`/`knight_sword`)
`factory_hero.rs` keeps only `KnightCfg`, `KnightParts`, `HeroFactory`, and `create_knight`. Remove `spawn_knight`, `spawn_effects`, and `knight_sword`; the scene assembles the knight subtree itself with the entity store.

## Update crate exports (`lib.rs`, `entity/mod.rs`, `entity/player/mod.rs`)
In `src/lib.rs`, export `Player` and `PlayerFactory`. Keep `pub mod player;` in `src/entity/mod.rs`. In `src/entity/player/mod.rs`, declare `mod factory_player;` and re-export `Player` and `PlayerFactory`.

## Rework `asset_preview` scene and systems to query the store fluently
In `src/scene/asset_preview/`:
- `scene.rs`: build the knight subtree via `HeroFactory::create_knight` + `store.add`, then add the root via `store.add(PlayerFactory::spawn(0), &[ChildSource::Existing(knight)])`. `rebuild_player`, `set_knight_idle`, and repositioning navigate via `store.all::<Player>().find(|p| p.index == 0)` then `get_child::<Knight>(&player)` (and `get_child_mut` for mutation).
- `sys_knight_controls.rs`, `sys_attack_effects.rs`, `sys_offsets.rs`: keep `player_index`; replace `Player::find(store, idx)` / `Player::class_of` with direct fluent queries, e.g. `store.all::<Player>().find(|p| p.index == self.player_index).and_then(|p| store.get_child::<Knight>(&p))`.

## Rework `battle_test` scene and systems to query the store fluently
In `src/scene/battle_test/`:
- `scene.rs`: build the knight subtree + `Player` root as in `asset_preview`; locate the knight for color/positioning via `store.all::<Player>().find(|p| p.index == 0)` + `get_child::<Knight>`.
- `sys_dash.rs`, `sys_enemy_ai.rs`, `sys_facing_lock.rs`, `sys_hud.rs`, `sys_orb.rs`: keep `player_index`; replace `Player::find`/`class_of` with direct fluent store queries.

## Build and run to verify
Run `cargo build` and `cargo run -p heroes-of-ironhold-desktop` (or `cargo check`) to confirm the code compiles and both scenes still spawn and animate the knight.

# Open Questions
None.

# Out of Scope
- We are not changing `src/util/estore.rs` or the `pico-entity-store` crate.
- We are not introducing generic multi-class hero traversal (`Player::class_of`) or multi-player systems yet.
- We are not touching unrelated recent work (dash system, enemy AI, skills bar, save/load).
