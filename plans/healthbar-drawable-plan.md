# Description

Make `HealthBar` a `Drawable` with `draw()` and `zdx()`, integrated into `DrawSystem`'s z-sorted draw batch. This removes the separate `HealthBarSystem` draw pass so health bars participate in the same z-sorted draw as `Animation` and `StaticImage`.

# TODO

- [ ] Add `visible`, `z_idx`, `position`, `track`, `fill` fields to `HealthBar`
- [ ] Move texture generation and draw constants from `sys_health_bar.rs` to `health_bar.rs`
- [ ] Implement `Drawable` for `HealthBar` (`draw()` and `zdx()`)
- [ ] Update `DrawSystem` to collect `HealthBar`
- [ ] Convert `HealthBarSystem` from `Draw` to `Update` (compute position + z_idx each frame)
- [ ] Update `scene.rs`: register `HealthBarSystem` as update (after `ZSortSystem`), remove from draw list

# TODO Explanation

## Item 1: Add fields to HealthBar

Add `visible: bool`, `z_idx: f32`, `position: Vec2`, `track: Texture2D`, `fill: Texture2D` to `HealthBar`. These store the pre-computed draw state so `draw()` is self-contained (matches how `Animation` owns its texture and position).

## Item 2: Move texture generation to health_bar.rs

Move `track_texture()`, `fill_texture()`, `make_texture()`, `sd_round_box()`, `health_color()`, and the constants (`BORDER`, `TRACK_RADIUS`, `FILL_RADIUS`, `BORDER_COLOR`, `BACKGROUND_COLOR`) from `sys_health_bar.rs` into `health_bar.rs`. `HealthBar::new()` creates the textures at construction time. `GAP_ABOVE_HEAD` stays in `sys_health_bar.rs` (position computation).

## Item 3: Implement Drawable for HealthBar

`HealthBar` implements `Drawable::draw()` (draws track texture at `self.position`, then fill texture clipped to `percent` width) and `zdx()` (returns `self.z_idx`). The draw logic currently in `HealthBarSystem::draw()` moves here.

## Item 4: Update DrawSystem

Add `HealthBar` variant to `DrawKind` enum. In `draw()`, collect all `HealthBar` components into the sorted `cmds` batch alongside `Animation` and `StaticImage`. Sort by `zdx()`, draw via `Drawable`.

## Item 5: Convert HealthBarSystem from Draw to Update

Change `impl Draw` to `impl Update`. Two-pass approach: first pass collects `(EntityRef, width, height, Rect, parent_z_idx)` tuples via immutable store access; second pass writes `position` and `z_idx` back via `store.update()`. Position is computed from the parent `RamHead`'s `Animation` rect (centered x, `rect.y - GAP_ABOVE_HEAD - height`). `z_idx` is set to `parent_z_idx + 0.5` (above the parent's visual parts).

## Item 6: Update scene.rs registration

Move `HealthBarSystem::new(store.clone())` from `add_draw` to `add_update`, registered after `ZSortSystem` so the parent's `z_idx` is already assigned. Remove the `add_draw(HealthBarSystem)` line.

# Objects

## HealthBar (modify)

`src/entity/health_bar.rs` — presentation-only health bar, now self-drawing.

```rust
#[derive(Clone, Debug)]
pub struct HealthBar {
    // ... existing fields ...
    pub visible: bool,       // whether to draw; skipped by DrawSystem when false
    pub z_idx: f32,          // assigned by HealthBarSystem update each frame
    pub position: Vec2,      // computed from parent RamHead's Animation rect
    pub track: Texture2D,    // pre-rendered border + background texture
    pub fill: Texture2D,     // pre-rendered fill gradient texture (tinted at draw)
}
```

## DrawKind (modify)

`src/systems/sys_draw.rs` — enum of drawable component variants.

```rust
enum DrawKind {
    Animation(Animation),
    Static(StaticImage),
    Bar(HealthBar),          // added
}
```

# Services

## HealthBarSystem (modify)

`src/systems/sys_health_bar.rs` — converted from `Draw` to `Update`; computes position and z_idx each frame.

```rust
pub struct HealthBarSystem {
    store: Rc<EStore>,
}

impl HealthBarSystem {
    /// Constructs the system with the shared entity store.
    pub fn new(store: Rc<EStore>) -> Self { ... }
}

impl Update for HealthBarSystem {
    /// Resolves parent RamHead Animation rect, writes position and z_idx to each HealthBar.
    fn update(&mut self, _ctx: &mut Context) { ... }
}
```

## DrawSystem (modify)

`src/systems/sys_draw.rs` — draws all `Drawable` components sorted by z-index.

```rust
impl Draw for DrawSystem {
    fn draw(&self, _ctx: &Context) {
        // ... existing Animation + StaticImage collection ...

        for bar in self.store.all::<HealthBar>() {
            if !bar.visible {
                continue;
            }
            cmds.push(DrawCmd {
                z_idx: bar.zdx(),
                kind: DrawKind::Bar(bar.clone()),
            });
        }

        // ... sort + draw loop (add DrawKind::Bar arm) ...
    }
}
```

# Open Questions

None

# Out of Scope

- Shared textures across `HealthBar` instances (each instance owns its own `Texture2D` for now; a single RamHead means one health bar)
- `ZSortSystem` integration for `HealthBar` (position and z_idx are computed independently by `HealthBarSystem`)
