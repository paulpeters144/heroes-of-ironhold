# Module & Import Rules

- Use the file-based module style: `src/entity.rs` alongside `src/entity/`. Never use `mod.rs`.
- Every type has one canonical import path. Sub-modules are private (`mod`) or `pub(crate) mod` only when they expose crate-shared constants used by other modules. Types are re-exported through the barrel file so external consumers never reach into deep paths.
- `lib.rs` re-exports only infrastructure (`Config`, `EStore`, `EventBus`, `Assets`, `DiContainer`, id namespaces, game-state types). Domain types are never re-exported from `lib.rs`.
- `src/prelude.rs` groups the types every system needs (`Context`, `EStore`, `EventBus`, `SubCollection`, core entity types, `System`, `SystemAgg`). Systems do `use crate::prelude::*;`.
- Barrel files (`entity.rs`, `scene.rs`, `util.rs`, `access.rs`, `systems.rs`) are pure re-exports — no logic, no types, no functions.

## Do: import from the barrel or prelude

```rust
// Systems: prelude covers the common set
use crate::prelude::*;

// Domain types: one level of nesting through the barrel
use crate::entity::{Knight, RamHead, Skill};
use crate::access::GameFont;
use crate::util::{did_attack, image_data_for};
use crate::events::EnemyDeathEvent;
```

## Not: deep paths to private sub-modules from outside the crate

```rust
// Don't reach into a sub-module from another module tree
use crate::entity::knight::Knight;            // no — use crate::entity::Knight
use crate::entity::animation::Animation;       // no — use crate::entity::Animation or prelude
use crate::util::attack::did_attack;           // no — use crate::util::did_attack
use crate::access::ids::font::GameFont;        // no — use crate::access::GameFont
```

## Do: keep sub-modules private, expose through the barrel

```rust
// src/entity.rs
mod animation;
mod attack_rect;
// ...

pub use animation::Animation;
pub use attack_rect::AttackRect;
```

## Not: `pub mod` on every sub-module

```rust
// src/entity.rs
pub mod animation;     // no — makes crate::entity::animation::Animation a second valid path
pub mod attack_rect;   // no
```

## When `pub(crate) mod` is acceptable

A sub-module stays `pub(crate)` only when it exposes constants or types that other modules within the crate need by their full path — typically because two sibling modules define the same name (e.g. `IDLE_FRAME` in both `knight` and `enemy`) and flattening them into the barrel would collide.

```rust
// src/entity.rs
pub(crate) mod knight;   // scene systems use crate::entity::knight::IDLE_FRAME
pub(crate) mod enemy;    // scene systems use crate::entity::enemy::IDLE_FRAME
```

## Barrel files stay pure

A barrel file contains only `mod` declarations and `pub use` re-exports. Orchestration logic, type definitions, and `impl` blocks belong in sibling sub-modules.

## Do: split logic into a sibling module

```rust
// src/scene.rs — pure barrel
mod transition;
pub use transition::{ChangeSceneEvent, LoadingScene, SceneFuture};
```

## Not: mix barrel and logic in one file

```rust
// src/scene.rs
mod transition;
pub use transition::SceneFuture;

pub struct LoadingScene { ... }       // no — belongs in scene/transition.rs
impl Scene for LoadingScene { ... }   // no
```
