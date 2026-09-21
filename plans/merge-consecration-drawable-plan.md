# Description

Merge the split `ConsecrationSystem` and `ConsecrationAuraSystem` back into a single `ConsecrationSystem` by making the ground effect and aura shader into drawable components with z-indices. `DrawSystem` handles the ordering — ground renders behind the knight (low z), aura renders on top (high z).

# TODO

- [ ] Create `ConsecrationGround` drawable component
- [ ] Create `ConsecrationAura` drawable component
- [ ] Add both to `DrawSystem` iteration
- [ ] Refactor `ConsecrationSystem` to manage components instead of drawing directly
- [ ] Remove `ConsecrationAuraSystem` and `sys_consecration_aura.rs`
- [ ] Update scene wiring

# TODO Explanation

## Create `ConsecrationGround` drawable component

A new component in `src/entity/consecration_ground.rs` that holds all the visual state for the ground effect: center position, area life, sparks, shards, shard timer, appear/fade state. Implements `Drawable` with a low `z_idx` (e.g., knight's z - 10) so it renders behind the knight. The `draw()` method contains all the procedural rendering: hexagon ground sheen, rune circles, burst sparks, diamond shards.

## Create `ConsecrationAura` drawable component

A new component in `src/entity/consecration_aura.rs` that holds the aura shader material and knight reference. Implements `Drawable` with a high `z_idx` (e.g., knight's z + 10) so it renders on top of the knight. The `draw()` method applies the golden glow shader to the knight's body, shield, and sword.

## Add both to `DrawSystem` iteration

Modify `src/systems/sys_draw.rs` to iterate `all::<ConsecrationGround>()` and `all::<ConsecrationAura>()`, adding them to the sorted draw commands alongside other drawables.

## Refactor `ConsecrationSystem` to manage components instead of drawing directly

The system's `update()` method spawns/updates `ConsecrationGround` and `ConsecrationAura` components based on the skill state. The `draw()` method becomes empty — all rendering happens through the drawable components. The system still handles: casting logic, area spawning, particle updates, component lifecycle.

## Remove `ConsecrationAuraSystem` and `sys_consecration_aura.rs`

Delete the separate aura system file and its module declaration. The aura logic moves into `ConsecrationAura` component's `draw()` method.

## Update scene wiring

Remove `ConsecrationAuraSystem` registration from `src/scene/battle_test/scene.rs`. `ConsecrationSystem` stays registered before `DrawSystem` (it only updates state, doesn't draw).

# Objects

## ConsecrationGround (new)

A drawable component holding the ground effect visual state. Lives in `src/entity/consecration_ground.rs`.

```rust
#[derive(Clone, Debug)]
pub struct ConsecrationGround {
    pub center: Vec2,                        // center of the hexagonal area
    pub area_life: f32,                      // time since area spawned (seconds)
    pub sparks: Vec<Spark>,                  // diamond burst sparks on cast
    pub shards: Vec<RuneShard>,              // rising diamond shard particles
    pub shard_timer: f32,                    // time until next shard spawn
    pub z_idx: f32,                          // draw order (low = behind knight)
    pub visible: bool,                       // whether to render
}
```

## ConsecrationAura (new)

A drawable component holding the golden aura shader state. Lives in `src/entity/consecration_aura.rs`.

```rust
#[derive(Clone)]
pub struct ConsecrationAura {
    pub knight_id: u64,                      // id of the knight to apply aura to
    pub material: Option<Material>,          // golden glow shader material
    pub z_idx: f32,                          // draw order (high = on top of knight)
    pub visible: bool,                       // whether to render
}
```

## Spark (new)

Particle data for burst sparks. Lives in `src/entity/consecration_ground.rs`.

```rust
#[derive(Clone, Debug)]
pub struct Spark {
    pub pos: Vec2,                           // current position
    pub vel: Vec2,                           // velocity
    pub size: f32,                           // diamond size
    pub life: f32,                           // remaining life (seconds)
}
```

## RuneShard (new)

Particle data for rising diamond shards. Lives in `src/entity/consecration_ground.rs`.

```rust
#[derive(Clone, Debug)]
pub struct RuneShard {
    pub pos: Vec2,                           // current position
    pub vel: Vec2,                           // velocity
    pub size: f32,                           // diamond size
    pub rotation: f32,                       // current rotation angle
    pub rot_speed: f32,                      // rotation speed
    pub life: f32,                           // remaining life (seconds)
    pub max_life: f32,                       // initial life for fade calc
}
```

## ConsecrationAura (modify)

Needs `Debug` derive removed or custom impl since `Material` doesn't implement `Debug`.

```rust
pub struct ConsecrationAura {
    // ... existing fields ...
}

impl std::fmt::Debug for ConsecrationAura { ... }  // custom impl skipping material
```

## ConsecrationSystem (modify)

Lives in `src/scene/battle_test/sys_consecration.rs`.

```rust
pub struct ConsecrationSystem {
    // ... existing fields ...
    ground_ref: Option<EntityRef>,           // ref to spawned ConsecrationGround
    aura_ref: Option<EntityRef>,             // ref to spawned ConsecrationAura
    // removed: direct drawing state (sparks, shards, etc. move to ConsecrationGround)
}
```

## ConsecrationAuraSystem (remove)

Removed from `src/scene/battle_test/sys_consecration_aura.rs`. The aura logic moves into `ConsecrationAura::draw()`. Callers (scene.rs) remove the registration.

## DrawSystem (modify)

Lives in `src/systems/sys_draw.rs`.

```rust
impl System for DrawSystem {
    fn draw(&self, _ctx: &Context) {
        // ... existing iteration ...

        for ground in self.store.all::<ConsecrationGround>() {
            if !ground.visible { continue; }
            cmds.push(DrawCmd {
                z_idx: ground.z_idx,
                kind: DrawKind::ConsecrationGround(ground.clone()),
            });
        }

        for aura in self.store.all::<ConsecrationAura>() {
            if !aura.visible { continue; }
            cmds.push(DrawCmd {
                z_idx: aura.z_idx,
                kind: DrawKind::ConsecrationAura(aura.clone()),
            });
        }

        // ... sort and draw ...
    }
}
```

## DrawKind (modify)

Lives in `src/systems/sys_draw.rs`.

```rust
enum DrawKind {
    // ... existing variants ...
    ConsecrationGround(ConsecrationGround),  // added
    ConsecrationAura(ConsecrationAura),      // added
}
```

# Services

## ConsecrationGround (new)

Drawable implementation in `src/entity/consecration_ground.rs`.

```rust
impl Drawable for ConsecrationGround {
    /// Renders the hexagonal ground effect: sheen, rune circles, sparks, shards.
    fn draw(&self) { ... }

    /// Returns the z-index for draw ordering (low = behind knight).
    fn zdx(&self) -> f32 { ... }
}
```

## ConsecrationAura (new)

Drawable implementation in `src/entity/consecration_aura.rs`.

```rust
impl Drawable for ConsecrationAura {
    /// Renders the golden aura shader over the knight's body, shield, and sword.
    fn draw(&self) { ... }

    /// Returns the z-index for draw ordering (high = on top of knight).
    fn zdx(&self) -> f32 { ... }
}
```

## ConsecrationSystem (modify)

Lives in `src/scene/battle_test/sys_consecration.rs`.

```rust
impl ConsecrationSystem {
    /// Creates the system with store, bus, and assets.
    pub fn new(store: Rc<EStore>, bus: Rc<EventBus>, assets: &Assets) -> Self { ... }
}

impl System for ConsecrationSystem {
    /// Updates particle state and manages ConsecrationGround/Aura component lifecycle.
    fn update(&mut self, ctx: &mut Context) { ... }

    /// Empty — all rendering happens through drawable components.
    fn draw(&self, _ctx: &Context) { ... }
}
```

# Open Questions

None — all resolved.

**Resolved:**
- **z_idx for ConsecrationGround**: Use `knight.z_idx - 10` to ensure it renders behind the knight while respecting scene depth ordering.
- **z_idx for ConsecrationAura**: Use `knight.z_idx + 10` to ensure it renders on top of the knight without interfering with UI elements.
- **Entity hierarchy**: `ConsecrationGround` is a standalone entity (stays where cast, doesn't follow knight). `ConsecrationAura` is a child of the knight entity (follows the knight as it moves).

# Out of Scope

- Changing the visual appearance of the ground effect or aura
- Modifying the shader code
- Changing how other skills handle z-ordering
