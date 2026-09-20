# Description

Overhaul the module and import structure to follow modern idiomatic Rust conventions. The current codebase has four problems: (1) every directory uses `mod.rs` instead of the modern file-based module style, (2) `lib.rs` re-exports 40+ types at the crate root while also exposing `pub mod entity`, creating two valid import paths for every type with no clear convention on which to use, (3) `scene/mod.rs` mixes barrel-file re-exports with 80 lines of orchestration logic, and (4) five systems in `asset_preview` are imported cross-scene by `battle_test`, violating the project's own rules and forcing `pub mod` visibility where `mod` should suffice.

# TODO

- [ ] 1. Switch all directory modules from `mod.rs` to file-based style
- [ ] 2. Replace the flat crate-root re-exports with a `prelude` module and a small set of infrastructure re-exports
- [ ] 3. Establish one canonical import path per type — make `entity` sub-modules private, expose types only through barrel `pub use`
- [ ] 4. Split `scene/mod.rs` — move `LoadingScene`, `SceneFuture`, `SceneState`, `ChangeSceneEvent`, and the `RawWakerVTable` into `scene/transition.rs`
- [ ] 5. Move `AnimationUpdateSystem` from `asset_preview/sys_animation.rs` and `menu/sys_animation.rs` into `src/systems/sys_animation.rs`
- [ ] 6. Copy the five knight systems from `asset_preview/` into `battle_test/`, make `asset_preview/`'s copies private (`mod`)
- [ ] 7. Make `util` sub-modules private (`mod`), keep selective `pub use` re-exports in `util/mod.rs`
- [ ] 8. Fix missing `EnemyDeathEvent` re-export in `lib.rs`
- [ ] 9. Consolidate identical client input-mapping loops into a single function in the core crate

# TODO Explanation

## 1. Switch all directory modules from `mod.rs` to file-based style

Every directory module currently uses `mod.rs` (e.g. `src/entity/mod.rs`). Modern Rust (edition 2018+) prefers the file-based style: `src/entity.rs` alongside `src/entity/`. This removes a level of indirection when navigating the tree — `src/entity.rs` is the module root, and `src/entity/knight.rs` is its child, with no `mod.rs` files anywhere.

Affected directories: `access/`, `di/`, `entity/`, `input/`, `manager/`, `scene/`, `systems/`, `ui/`, `util/`, and all sub-directories within `entity/` (`dash/`, `enemy/`, `hero/`, `knight/`, `player/`, `skills/`) and `scene/` (`asset_preview/`, `battle_test/`, `menu/`).

## 2. Replace the flat crate-root re-exports with a `prelude` module

`lib.rs` currently re-exports 40+ types from deep module paths (`Animation`, `Knight`, `Skill`, `HealthBar`, etc.) while also declaring `pub mod entity`. This means `crate::Animation` and `crate::entity::animation::Animation` are both valid, and different files use different paths with no rule governing which.

The fix: keep only infrastructure re-exports at the crate root (`Config`, `EStore`, `EventBus`, `Assets`, `DiContainer`, `Game`, `Context`, entry-point functions). Add a `prelude` module that groups the types every system needs (`Animation`, `Context`, `EStore`, `EventBus`, `SubCollection`, `AttackRect`, `CollisionRect`, `Drawable`, `System`, `SystemAgg`). Consumers do `use crate::prelude::*;` for the common set, and `use crate::entity::knight::Knight;` for domain types.

## 3. Establish one canonical import path per type

Currently `entity/mod.rs` declares all sub-modules as `pub mod`, meaning every type is reachable via its deep path *and* via the crate-root re-export. The fix: make entity sub-modules private (`mod`), and let each module's barrel file (`entity.rs`) control the public surface via `pub use`. This means the only valid import path for `Knight` is `crate::entity::Knight` (from the barrel) or `crate::prelude::Knight` — never `crate::entity::knight::Knight` from outside the crate.

Inside the crate, the convention becomes: `use crate::entity::{Knight, RamHead, Skill};` — one level of nesting, no deep paths, no ambiguity.

## 4. Split `scene/mod.rs`

`scene/mod.rs` is 110 lines — 12 lines of barrel re-exports and 98 lines of orchestration types (`ChangeSceneEvent`, `LoadingScene`, `SceneFuture`, `SceneState`, `RawWakerVTable`, `dummy_waker`). Move the orchestration types into `scene/transition.rs`, leaving `scene/mod.rs` (or `scene.rs` after step 1) as a pure barrel file.

## 5. Move `AnimationUpdateSystem` to shared systems

`asset_preview/sys_animation.rs` and `menu/sys_animation.rs` are character-for-character identical (56 lines each). Move the system into `src/systems/sys_animation.rs` as a shared system, delete both copies, and register it from each scene via `use crate::systems::AnimationUpdateSystem`.

## 6. Copy knight systems into `battle_test/`, close `asset_preview/` visibility

`battle_test/scene.rs` imports five systems from `asset_preview/` (lines 24-28), violating `rules/systems.md`. Per the rules, each scene gets its own copy. Copy the five files (`sys_knight_attack.rs`, `sys_knight_attack_effects.rs`, `sys_knight_controls.rs`, `sys_knight_offsets.rs`, and the now-shared `sys_animation.rs` from step 5) into `battle_test/`, adjust their module paths, and change `asset_preview/mod.rs` to declare all its systems as `mod` (private).

## 7. Make `util` sub-modules private

`util/mod.rs` declares all sub-modules as `pub mod`, giving two valid paths for `did_attack` (`crate::util::attack::did_attack` and `crate::util::did_attack`). Internal code uses the deep path. Fix: change to `mod` (private), keep the `pub use` re-exports. This makes `crate::util::did_attack` the only valid path.

## 8. Fix missing `EnemyDeathEvent` re-export

`events.rs` defines `EnemyDeathEvent` (line 30) but `lib.rs` line 41 does not include it in the re-export list. Either add it to the prelude or (better) let the `events` barrel handle it consistently.

## 9. Consolidate client input-mapping loops

`clients/desktop/src/main.rs` and `clients/web/src/main.rs` are 26 lines of character-for-character identical code. Extract the input-mapping logic into a function like `pub fn poll_input()` in the core crate's `input` module, so each client's `main.rs` becomes a 5-line loop.

# Objects

## prelude (new)

A new module at `src/prelude.rs` that re-exports the types every system and scene needs, so consumers can `use crate::prelude::*;` instead of assembling imports from multiple paths.

```rust
// src/prelude.rs

// Infrastructure
pub use crate::{Context, EStore, EventBus};
pub use event_bus::SubCollection;

// Core entity types used by nearly every system
pub use crate::entity::animation::Animation;
pub use crate::entity::attack_rect::AttackRect;
pub use crate::entity::collision_rect::CollisionRect;
pub use crate::entity::drawable::Drawable;
pub use crate::entity::floating_text::FloatingText;
pub use crate::entity::static_image::StaticImage;

// System traits
pub use crate::systems::{System, SystemAgg};
```

## Lib (modify)

`src/lib.rs` — strip the 30+ entity/event re-exports, keep only infrastructure and entry points, add `pub mod prelude`.

```rust
// src/lib.rs

mod access;
mod di;
pub mod entity;      // changed: sub-modules become private, barrel controls surface
pub mod events;
pub mod input;
pub mod manager;
pub mod prelude;     // added
pub mod scene;
pub mod systems;
pub mod ui;
mod util;            // changed: was pub mod, now private

// Infrastructure re-exports only
pub use di::DiContainer;
pub use util::config::Config;
pub use util::estore::EStore;
pub use util::event_bus::EventBus;
pub use access::assets::Assets;

// Entry points
pub use manager::{draw, init, update, window_conf};

// Game state types used by clients
pub use lib::{Context, Game};  // existing, unchanged

// ... existing init(), update(), draw() functions unchanged ...
```

## Entity (modify)

`src/entity.rs` (renamed from `src/entity/mod.rs`) — switch to file-based style, make sub-modules private, add barrel re-exports so `crate::entity::Knight` works.

```rust
// src/entity.rs (was src/entity/mod.rs)

mod animation;
mod area_rect;
mod attack_rect;
mod collision_rect;
mod dash;
mod drawable;
mod enemy;
mod factory_enemy;
mod factory_hero;
mod factory_skills;
mod floating_text;
mod health_bar;
mod hero;
mod impact_frame;
mod knight;
mod player;
mod skills;
mod static_image;

// Barrel re-exports — the canonical import path for entity types
pub use animation::Animation;
pub use area_rect::AreaRect;
pub use attack_rect::AttackRect;
pub use collision_rect::CollisionRect;
pub use dash::{Dash, DashCfg};
pub use drawable::Drawable;
pub use enemy::{EnemyStats, RamHead};
pub use factory_enemy::EnemyFactory;
pub use factory_hero::{HeroFactory, KnightCfg, KnightParts};
pub use factory_skills::{SkillSlotCfg, SkillSlotParts, SkillsFactory, SkillsParts};
pub use floating_text::FloatingText;
pub use health_bar::HealthBar;
pub use hero::{CoreStat, HeroStats};
pub use impact_frame::ImpactFrame;
pub use knight::{
    Effect, EffectKind, FrameOffsets, GuardianShield, Knight, KnightLock, Shield, Sword,
};
pub use player::{PlayerFactory, PlayerOne, PlayerTwo};
pub use skills::{
    LastUsedSkill, Skill, SkillDirection, SkillIcon, SkillIconKind, SkillsWidget,
};
pub use static_image::StaticImage;
```

## Events (modify)

`src/events.rs` — no structural change, but the missing `EnemyDeathEvent` is now reachable through the `events` module directly (consumers do `use crate::events::EnemyDeathEvent`).

```rust
// src/events.rs — unchanged content, just noting EnemyDeathEvent is now consistently reachable

// ... existing event structs ...

#[derive(Clone, Debug)]
pub struct EnemyDeathEvent {
    pub enemy: u64,
}

// ... existing event structs ...
```

## Util (modify)

`src/util.rs` (renamed from `src/util/mod.rs`) — switch to file-based style, make sub-modules private.

```rust
// src/util.rs (was src/util/mod.rs)

mod attack;
mod camera;
mod config;
mod estore;
mod event_bus;
mod view_scale;

pub use attack::{did_attack, image_data_for, ImageData};
pub(crate) use camera::GameCamera;
pub(crate) use config::Config;
pub(crate) use estore::EStore;
pub(crate) use event_bus::EventBus;
```

## Access (modify)

`src/access.rs` (renamed from `src/access/mod.rs`) — switch to file-based style, make sub-modules private.

```rust
// src/access.rs (was src/access/mod.rs)

mod assets;
mod font;
mod game_state;
mod ids;

pub use assets::Assets;
pub use font::{FontTag, GameFont, TextStyle};
pub use game_state::{DiskJsonStore, GameState, GameStateStore, SaveError};
pub use ids::{file, font, images, shader, sound, texture};
```

## Transition (new)

`src/scene/transition.rs` — extracted from the bottom 98 lines of `scene/mod.rs`.

```rust
// src/scene/transition.rs

use crate::scene::Scene;
use crate::{Config, Context};
use macroquad::prelude::*;
use std::future::Future;
use std::pin::Pin;
use std::ptr;
use std::rc::Rc;
use std::task::{Context as TaskContext, Poll, RawWaker, RawWakerVTable, Waker};

pub struct ChangeSceneEvent(pub super::SceneId);

pub struct LoadingScene {
    cfg: Rc<Config>,
}

// ... existing LoadingScene impl and Scene impl ...

pub struct SceneFuture {
    future: Pin<Box<dyn Future<Output = Box<dyn Scene>>>>,
    result: Option<Box<dyn Scene>>,
}

// ... existing SceneFuture impl ...

pub(crate) enum SceneState {
    Idle {
        scene: Box<dyn Scene>,
    },
    Transition {
        scene: Box<dyn Scene>,
        next_scene_loader: SceneFuture,
        elapsed: f32,
        min_wait: f32,
    },
}
```

## Scene (modify)

`src/scene.rs` (renamed from `src/scene/mod.rs`) — stripped down to a pure barrel file after extracting transition logic.

```rust
// src/scene.rs (was src/scene/mod.rs)

mod asset_preview;
mod battle_test;
mod factory;
mod menu;
mod traits;
mod transition;

pub use asset_preview::AssetPreviewScene;
pub use battle_test::BattleTestScene;
pub use factory::{SceneFactory, SceneId};
pub use menu::MenuScene;
pub use traits::Scene;
pub use transition::{ChangeSceneEvent, LoadingScene, SceneFuture, SceneState};
```

## AssetPreview (modify)

`src/scene/asset_preview.rs` (renamed from `src/scene/asset_preview/mod.rs`) — all systems become private after step 6.

```rust
// src/scene/asset_preview.rs (was src/scene/asset_preview/mod.rs)

mod scene;
mod sys_animation;         // changed: was pub mod, now mod
mod sys_knight_attack;     // changed: was pub mod, now mod
mod sys_knight_attack_effects;     // changed: was pub mod, now mod
mod sys_knight_controls;   // changed: was pub mod, now mod
mod sys_knight_offsets;    // changed: was pub mod, now mod

pub use scene::AssetPreviewScene;
```

## BattleTest (modify)

`src/scene/battle_test.rs` (renamed from `src/scene/battle_test/mod.rs`) — gains five new system modules (copies from `asset_preview`).

```rust
// src/scene/battle_test.rs (was src/scene/battle_test/mod.rs)

mod scene;
mod sys_camera;
mod sys_divine_area;
mod sys_enemy_death;
mod sys_guardian_shield;
mod sys_hud;
mod sys_knight_attack;             // added: copy from asset_preview
mod sys_knight_attack_effects;     // added: copy from asset_preview
mod sys_knight_attack_hit;
mod sys_knight_controls;           // added: copy from asset_preview
mod sys_knight_dash;
mod sys_knight_offsets;            // added: copy from asset_preview
mod sys_map_draw;
mod sys_orb;
mod sys_ramhead_ai;
mod sys_shield_cycle;
mod sys_skill;
mod sys_swords_skill;

pub use scene::BattleTestScene;
```

## Systems (modify)

`src/systems.rs` (renamed from `src/systems/mod.rs`) — gains `sys_animation` (moved from scenes).

```rust
// src/systems.rs (was src/systems/mod.rs)

mod sys_aggregate;
mod sys_animation;         // added: moved from scene sub-modules
mod sys_collision_rect;
mod sys_debug_draw;
mod sys_draw;
mod sys_health_bar;
mod sys_health_text_animation;
mod sys_hit_reaction;
mod sys_knight_combat;
mod sys_z_sort;

pub use sys_aggregate::{System, SystemAgg};
pub use sys_animation::AnimationUpdateSystem;     // added
pub use sys_collision_rect::CollisionRectSystem;
pub use sys_debug_draw::DebugDrawSystem;
pub use sys_draw::DrawSystem;
pub use sys_health_bar::HealthBarSystem;
pub use sys_health_text_animation::HealthTextAnimationSystem;
pub use sys_hit_reaction::HitReactionSystem;
pub use sys_knight_combat::KnightCombatSystem;
pub use sys_z_sort::ZSortSystem;
```

## Menu (modify)

`src/scene/menu.rs` (renamed from `src/scene/menu/mod.rs`) — drops its copy of `sys_animation.rs`.

```rust
// src/scene/menu.rs (was src/scene/menu/mod.rs)

mod scene;
mod sys_menu_input;
// sys_animation removed — now uses crate::systems::AnimationUpdateSystem

pub use scene::MenuScene;
```

## Input (modify)

`src/input.rs` (renamed from `src/input/mod.rs`) — gains `poll_input()` to consolidate client code.

```rust
// src/input.rs (was src/input/mod.rs)

// ... existing Input enum, set(), down(), down_once(), end_frame() ...

/// Read host key state and push it into the input buffer.
/// Called once per frame from the client's main loop.
pub fn poll_input() { ... }
```

# Services

## Input (modify)

`src/input.rs` — gains a `poll_input()` function that encapsulates the key-mapping loop currently duplicated across both client `main.rs` files.

```rust
impl /* free functions in input module */ {
    // ... existing functions ...

    /// Read all host key states and push them into the input buffer.
    /// Replaces the 12-line mapping loop in each client's main.rs.
    pub fn poll_input() { ... }
}
```

# Open Questions

- Should the `ids.rs` asset-ID enums (currently one 360-line file with 8 enums inside the `images` module) be split into per-category files (`ids/images/knight.rs`, `ids/images/enemy.rs`, etc.) as part of this work, or deferred to a follow-up?
- The `pub(crate)` re-exports in `entity/knight/mod.rs` (frame constants, movement speeds, attack phases) are used by systems in both `asset_preview` and `battle_test`. After the knight systems are copied into `battle_test`, should these stay `pub(crate)` or should they move to a shared location?
- Should `util::camera::GameCamera` remain re-exported at the crate root (it's used by the `Context` struct in `lib.rs`), or should `lib.rs` reference it via `util::camera::GameCamera` directly?

# Out of Scope

- We are not changing the workspace structure (the root-is-core pattern stays).
- We are not changing the `pico-entity-store`, `event-bus`, or `tiled` crates.
- We are not changing any system logic, game mechanics, or rendering code.
- We are not renaming any types, fields, or methods — only moving files and adjusting visibility.
- We are not changing the `rules/` files (though `rules/systems.md` may want a follow-up to clarify the shared-system exception for `AnimationUpdateSystem`).
