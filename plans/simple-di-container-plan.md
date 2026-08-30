# Description
Replace the current async DI (`crates/di-container`, used via `Container`/`ContainerBuilder`) with a simple, naive synchronous `DiContainer` struct in a new `src/di/` module. Singletons are owned as `Rc<T>` and exposed via accessor methods (`di.camera()`, `di.config()`, etc.); `assets()` is transient and returns a fresh `Assets` access on every call. No `Box::leak`, no `&'static`.

From `src/lib.rs:49-59`, the exact services registered today are: `Config`, `EventBus`, `Assets` (transient), `EStore`, `SystemAgg`, `GameRenderTarget`, `GameCamera`. The accessor set is derived from actual consumers (see below).

# TODO
- [ ] Create `src/di/mod.rs` with the `DiContainer` struct and `Rc`-based accessors
- [ ] Add plain constructors to `EventBus`, `GameRenderTarget`, and `GameCamera`
- [ ] Wire singletons in `DiContainer::new()` in dependency order
- [ ] Register `mod di;` in `src/lib.rs` and build the container in `init()`
- [ ] Update `Manager` and `SceneFactory` to hold `Rc` handles and use accessors
- [ ] Remove the old `Injectable`/`BuildContext` plumbing and `di_container` imports
- [ ] Verify with `cargo check` / `cargo build`

# TODO Explanation
## Create `src/di/mod.rs` with the `DiContainer` struct and `Rc`-based accessors
```rust
use std::rc::Rc;

pub struct DiContainer {
    config: Rc<Config>,
    event_bus: Rc<EventBus>,
    estore: Rc<EStore>,
    camera: Rc<GameCamera>,
    render_target: GameRenderTarget, // private; kept alive, no accessor
}

impl DiContainer {
    pub fn config(&self) -> Rc<Config>      { Rc::clone(&self.config) }
    pub fn event_bus(&self) -> Rc<EventBus> { Rc::clone(&self.event_bus) }
    pub fn estore(&self) -> Rc<EStore>      { Rc::clone(&self.estore) }
    pub fn camera(&self) -> Rc<GameCamera>  { Rc::clone(&self.camera) }
    pub fn assets(&self) -> Assets           { Assets::new() } // transient
}
```

Accessor set rationale (from actual consumers):
- `config()` — used by `Manager::new` (`manager/mod.rs:32`) and by `GameRenderTarget`/`GameCamera` construction.
- `event_bus()` — used by `Manager::new` (`manager/mod.rs:22`).
- `camera()` — used by `Manager::new` (`manager/mod.rs:23`).
- `estore()` — used by `SceneFactory::create` (`scene/factory.rs:32,44`).
- `assets()` — transient; used by `SceneFactory::create` (`scene/factory.rs:27,39`).
- `SystemAgg` is registered as a singleton (`lib.rs:54`) but **never resolved** via `get::<SystemAgg>()` — each scene builds its own `SystemAgg::new()` (`battle_test/scene.rs:32`, `asset_preview/scene.rs:62`). So no `system_agg()` accessor is needed; it is dropped from the container.
- `GameRenderTarget` is only consumed internally by `GameCamera` construction (`camera.rs:46`), never fetched by name elsewhere, so it needs no public accessor — but it is kept as a private owned field so the original render target stays alive for the program's lifetime (matching today's leaked-singleton behavior). Note: `GameCamera` already holds its own `RenderTarget` clone, so this could later be folded into `GameCamera::new` and the `GameRenderTarget` type removed entirely.

`Rc` is used (not `Arc`, not `&'static`) because the game is single-threaded: no `thread::spawn` in `src/`, macroquad handles are `!Send`, and every future here is non-`Send` (`Pin<Box<dyn Future + '_>>`). `Rc` is already the convention (`manager/mod.rs:15`, the `event-bus` crate). All current singletons already use interior mutability (`EventBus` inner state, `EStore` -> `PicoEntityStore`, `SystemAgg` -> `RefCell`), so plain `Rc<T>` suffices; a future singleton needing direct `&mut` would need `Rc<RefCell<T>>`.

## Add plain constructors
Currently these types have no `new()`; they are built inside `inject()`. Add:
- `EventBus::new()` -> `EventBus(InnerEventBus::new())` (`util/event_bus.rs:21`).
- `GameRenderTarget::new(config: &Config)` -> calls `render_target(w, h)` + `set_filter(Nearest)` (`util/camera.rs:15-22`).
- `GameCamera::new(config: &Config, rt: &GameRenderTarget)` -> computes `display_rect` and clones `rt.target` (`util/camera.rs:44-60`).

## Wire singletons in dependency order
`DiContainer::new()` builds, in order: `config = Config::default()` -> `render_target = GameRenderTarget::new(&config)` -> `camera = GameCamera::new(&config, &render_target)`; then `event_bus = EventBus::new()` and `estore = EStore::new()`. Wrap each shared singleton in `Rc::new(...)`. `render_target` stays an owned private field (not `Rc`) since nothing shares it.

## Register module and build the container
Add `mod di;` (plus `pub use di::DiContainer;` if desired) in `src/lib.rs`. In `init()`, replace the `ContainerBuilder` block with:
```rust
let di = Rc::new(DiContainer::new());
```
No leak. `Game { mgr: Manager::new(di), ... }` takes ownership of the `Rc` and clones handles into `Manager`/`SceneFactory` as needed. `DiContainer::new()` runs synchronously in `init()` (the old `inject()` bodies were already effectively synchronous; `render_target` only needs the macroquad context, which exists once `init()` runs).

## Update consumers
- `Manager::new(di: Rc<DiContainer>)`: `container.get::<EventBus>()` -> `di.event_bus()`, `get::<GameCamera>()` -> `di.camera()`, `get::<Config>()` -> `di.config()`. Change `Manager` fields from `&'static Config`/`&'static GameCamera` to `Rc<Config>`/`Rc<GameCamera>` (`manager/mod.rs:12-13`). The existing `Rc<Cell<Option<SceneId>>>` (`manager/mod.rs:15`) is unaffected.
- `SceneFactory::new(di: Rc<DiContainer>)`: field `container: &'static Container` -> `di: Rc<DiContainer>`. `create()` becomes synchronous: `self.container.resolve_transient::<Assets>().await` -> `self.di.assets()`, `self.container.get::<EStore>()` -> `self.di.estore()`, and `self.cfg` comes from `self.di.config()` (clone the `Rc`) when constructing scenes. Adjust the manager's loader (`manager/mod.rs:55-59`) to call `factory.create(id)` without `.await` (only `scene.load().await` remains async).

## Remove old plumbing
Delete the `Injectable` impls and `use di_container::{BuildContext, Injectable}` imports from `util/config.rs`, `util/event_bus.rs`, `util/camera.rs`, `util/estore.rs`, `access/assets.rs`, and `systems/system_agg.rs`. Remove the `di_container::ContainerBuilder` import from `lib.rs`.

Remove the `crates/di-container` crate (workspace members, dependency, and the `crates/di-container/` directory).

## Verify
Run `cargo check` (and `cargo build`) to confirm everything compiles after the swap.

# Out of Scope
- Deleting the `crates/di-container` crate (done: removed from workspace members and dependencies).
- No automatic dependency injection (no `Injectable` trait, no `BuildContext`, no reflection or resolution-by-type).
- No async resolution — `resolve_transient().await` is replaced by synchronous accessors.
- No `Arc` or threading; `Rc` is used to match the single-threaded game model.
- Not modifying `pico_entity_store`, `src/util/estore.rs`'s storage logic, or the `tiled` crate.
- `SystemAgg` is intentionally not exposed on the container (it is constructed per-scene today and registered-but-unused as a singleton).
