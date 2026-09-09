# Description
Remove the floating damage-number text that `HandleAttackSystem` currently draws when a RamHead is hit, and replace it with a dedicated, event-driven system. We introduce a `HealthChangeEvent` (carrying the affected entity's id, a signed amount, and its world-space `Rect`), a new `FloatingText` drawable component that renders text at a z-index, and a new `HealthTextAnimationSystem` (`sys_health_text_animation.rs`) that listens for the event and animates the text.

# TODO
- [ ] Add `HealthChangeEvent` to `src/events.rs`
- [ ] Add `FloatingText` component (drawable at a z-index) under `src/entity/`
- [ ] Create `HealthTextAnimationSystem` in `src/systems/sys_health_text_animation.rs`
- [ ] Wire `FloatingText` into `DrawSystem` so it renders in the z-sorted scene pass
- [ ] Modify `HandleAttackSystem` to remove text rendering and instead fire `HealthChangeEvent`
- [ ] Register the new system in the battle scene and export new symbols

# TODO Explanation

## Add `HealthChangeEvent` to `src/events.rs`
A small `Clone + Debug` struct following the existing `AttackEvent` pattern. It carries the entity id, the signed delta (`> 0` heal, `< 0` damage), and the world-space `Rect` of the entity that received the change. `Rect` comes from `macroquad::prelude`, so the module needs that import.

## Add `FloatingText` component
A presentation-only component mirroring `HealthBar`/`StaticImage`: it implements `Drawable` (`draw` + `zdx`) so `DrawSystem` can render it in the z-sorted pass. It owns its text, font, position, and animation state (age/lifetime/rise). The `HealthTextAnimationSystem` spawns these as entities and advances their animation; `DrawSystem` renders them. `FloatingText` entities are standalone (not children of the affected entity) and are assigned a fixed on-top `z_idx` constant so they render above scene entities. `draw()` keeps the black-outline treatment (8 offset black copies behind the colored glyphs) used by the current damage text.

## Create `HealthTextAnimationSystem`
An `Update`-only system (text is rendered by `DrawSystem`, not this system). It subscribes to `HealthChangeEvent` in `new()`, enqueues events, and in `update()` spawns a `FloatingText` per event (positioned above the event `rect`, color chosen by the sign of `amount` — one color for damage, another for healing) and ticks existing `FloatingText` entities forward (age += `ctx.dt`, drift upward, remove when expired). Font is extracted once in `new()` per the conventions rules.

## Wire `FloatingText` into `DrawSystem`
Add a `Text` variant to `DrawKind` and iterate `self.store.all::<FloatingText>()`, pushing each as a draw command sorted by `zdx()`. This is the mechanism that makes text respect z-index ordering.

## Modify `HandleAttackSystem`
Strip all text handling (the `FloatingNumber` struct, `damage_numbers`, the `font` field, the two `DAMAGE_TEXT_*` constants, and the text branch of `draw()`). Keep the white flash. When damage is applied to a target, fire a `HealthChangeEvent` (with the target's `Animation::rect()` and `amount = -(damage)` — the raw damage value, not clamped to remaining hp) instead of pushing a floating number. This requires storing the `Rc<EventBus>` on the struct.

## Register the new system and export new symbols
Add the module to `src/systems/mod.rs`, export `FloatingText` from `src/entity/mod.rs` and `src/lib.rs`, export `HealthChangeEvent` from `src/lib.rs`, and register `HealthTextAnimationSystem` in `src/scene/battle_test/scene.rs`. Register it **after** `HandleAttackSystem` (both live in `load()`), since `SystemAgg` runs updates in registration order and `HandleAttackSystem` fires the event that `HealthTextAnimationSystem` drains — ordering after avoids a one-frame delay between damage and its text.

# Objects

## HealthChangeEvent (new)
[Event fired when an entity's health changes; lives in `src/events.rs`.]

```rust
use macroquad::prelude::Rect;

#[derive(Clone, Debug)]
pub struct HealthChangeEvent {
    pub entity: u64,  // id of the entity whose health changed
    pub amount: i32,  // signed, raw (unclamped) delta: > 0 heal, < 0 damage; text renders as "-5"/"+12"
    pub rect: Rect,   // world-space size/position of the affected entity
}
```

## FloatingText (new)
[A drawable text component rendered at a z-index; lives in `src/entity/floating_text.rs`.]

```rust
use crate::Drawable;
use macroquad::prelude::{Color, Font, Vec2};

#[derive(Clone, Debug)]
pub struct FloatingText {
    pub text: String,      // label to render (e.g. "+12" or "-5")
    pub position: Vec2,    // world-space anchor (top-center above the entity rect)
    pub font: Font,        // macroquad font handle
    pub font_size: u16,    // rasterized glyph size
    pub color: Color,      // text color (chosen from the sign of the change)
    pub age: f32,          // seconds since spawn
    pub lifetime: f32,     // seconds until despawn
    pub rise_speed: f32,   // vertical drift in world units per second
    pub visible: bool,     // whether DrawSystem should render it
    pub z_idx: f32,        // draw order within the scene pass
}

impl FloatingText {
    /// Builds a new FloatingText anchored above `rect`.
    pub fn new(text: String, rect: Rect, font: Font, font_size: u16, color: Color) -> Self { ... }
}

impl Drawable for FloatingText {
    fn draw(&self) { ... }
    fn zdx(&self) -> f32 { ... }
}
```

## HandleAttackSystem (modify)
[Remove all text rendering and add `HealthChangeEvent` firing; lives in `src/systems/sys_handle_attack.rs`.]

```rust
#[derive(Clone)]
pub struct HandleAttackSystem {
    store: Rc<EStore>,
    bus: Rc<EventBus>,                             // added: held so update() can fire HealthChangeEvent
    flash_material: Rc<Option<Material>>,
    flashing: Rc<RefCell<HashMap<u64, f32>>>,
    queue: Rc<RefCell<VecDeque<AttackEvent>>>,
    _subs: Rc<SubCollection>,
    // damage_numbers: Rc<RefCell<Vec<FloatingNumber>>>, // removed
    // font: GameFont,                                    // removed
}
```

## FloatingNumber (remove)
[Private struct in `src/systems/sys_handle_attack.rs`; removed along with its usage in `update()`/`draw()` and the `DAMAGE_TEXT_LIFETIME` / `DAMAGE_TEXT_HEIGHT` constants.]

```rust
#[derive(Clone)]
struct FloatingNumber {
    pos: Vec2,
    text: String,
    remaining: f32,
}
```

## DrawSystem (modify)
[Gains a `Text` draw kind so `FloatingText` entities render in the z-sorted pass; lives in `src/systems/sys_draw.rs`.]

```rust
enum DrawKind {
    Animation(Animation),
    Static(StaticImage),
    Bar(HealthBar),
    Text(FloatingText), // added
}
```

# Services

## HealthTextAnimationSystem (new)
[Update system that spawns and animates `FloatingText` entities in response to `HealthChangeEvent`; lives in `src/systems/sys_health_text_animation.rs`.]

```rust
pub struct HealthTextAnimationSystem {
    store: Rc<EStore>,
    queue: Rc<RefCell<VecDeque<HealthChangeEvent>>>,
    font: Font,
    font_size: u16,
    _subs: Rc<SubCollection>,
}

impl HealthTextAnimationSystem {
    /// Subscribes to HealthChangeEvent and extracts the Pixellari font (size 24) once at construction.
    pub fn new(store: Rc<EStore>, bus: Rc<EventBus>, assets: &Assets) -> Self { ... }
}

impl Update for HealthTextAnimationSystem {
    fn update(&mut self, ctx: &mut Context) { ... }
}
```

# Open Questions


# Out of Scope
- We are not adding healing/health-increase gameplay mechanics; the system only reacts to whatever fires `HealthChangeEvent` (damage only for now).
- We are not changing `sys_health_bar.rs` or the `HealthBar` component.
- We are not modifying `ZSortSystem` (the text uses a fixed on-top `z_idx`).
- We are not changing `AttackEvent` or `AttackHitSystem`.
- We are not adding any audio feedback on health change.
- Exact animation motion (rise speed, lifetime, optional scale "pop") is deferred to a prototype pass; the plan only fixes the shape of the animation (rise + fade, black outline, colored by direction).
