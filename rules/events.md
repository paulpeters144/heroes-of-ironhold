# System Communication Rules

- Systems talk to each other only by emitting and listening to events on the shared `EventBus`. Never call another system's methods, reach into its fields, or wire two systems together with direct references. One system `fire`s an event; any other system that cares subscribes to it.

## Events are small, id-bearing structs in `src/events.rs`

Declare an event as a `Clone + Debug` struct in `src/events.rs`. Events carry entity ids (`u64`), not store guards, `EntityRef`s, or component data — the receiver resolves ids back into components when it processes the event.

```rust
#[derive(Clone, Debug)]
pub struct AttackEvent {
    pub attacker: u64,
    pub targets: Vec<u64>,
}
```

Any `'static` type is already an `Event` (blanket impl), so no extra trait impl is required.

## Do: emit from the system that detected the thing

The system holds `Rc<EventBus>` (passed in through `new`) and calls `fire` during `update`:

```rust
pub struct AttackHitSystem {
    store: Rc<EStore>,
    bus: Rc<EventBus>,
    prev_visible: bool,
}

impl Update for AttackHitSystem {
    fn update(&mut self, _ctx: &mut Context) {
        // ... detect the hit, collect target ids ...
        if area.visible && !self.prev_visible && !targets.is_empty() {
            self.bus.fire(&AttackEvent { attacker: knight.id(), targets });
        }
        self.prev_visible = area.visible;
    }
}
```

- Fire during `update()`, not from inside an event handler.
- An event may carry multiple targets (ids) in a single `fire` rather than firing once per target.

## Do: subscribe in `new()` and keep the subscription alive

The listener creates a `SubCollection`, attaches the handler, and stores the collection on the struct so the subscription stays registered. Dropping the collection disposes the subscriptions.

```rust
pub struct HandleAttackSystem {
    queue: Rc<RefCell<VecDeque<AttackEvent>>>,
    _subs: Rc<SubCollection>,
}

impl HandleAttackSystem {
    pub fn new(store: Rc<EStore>, bus: Rc<EventBus>) -> Self {
        let queue = Rc::new(RefCell::new(VecDeque::new()));
        let subs = Rc::new(SubCollection::new());
        let queue_for_handler = queue.clone();
        subs.on::<AttackEvent>(&bus, move |event: &AttackEvent| {
            queue_for_handler.borrow_mut().push_back(event.clone());
        });
        Self { queue, _subs: subs }
    }
}
```

- The handler must only enqueue the event. Do the real work later, in `update()`.
- The bus fires handlers synchronously, so keep handlers trivial (no heavy work, no firing nested events).

## Do: drain the queue and resolve ids in `update()`

```rust
impl Update for HandleAttackSystem {
    fn update(&mut self, _ctx: &mut Context) {
        while let Some(event) = self.queue.borrow_mut().pop_front() {
            for target in event.targets {
                if let Some(body) = self
                    .store
                    .get_by_id::<RamHead>(target)
                    .and_then(|e| self.store.get_child::<Animation>(&e))
                {
                    // ... act on the component ...
                }
            }
        }
    }
}
```

Resolve ids into components on the receiving side with `get_by_id` / the fluent chain (see `rules/store-access.md`). Events never carry the guard or the component itself.

## Wiring: one bus per app, threaded from the DI container

The `EventBus` is created once in the DI container and threaded down through the factory into each scene and system constructor:

- `DiContainer::event_bus()` returns `Rc<EventBus>`.
- `SceneFactory::create` passes `self.di.event_bus()` into the scene's `new`.
- The scene stores the bus and hands `Rc<EventBus>` to the systems that need it.

Do not construct a new `EventBus` per scene or per system.

## Not: direct system-to-system coupling

```rust
// Don't reach into another system to read its state or trigger behavior.
struct DamageSystem {
    attack_system: Rc<AttackSystem>, // no
}
```

```rust
// Don't do real work inside the event handler.
subs.on::<AttackEvent>(&bus, move |event| {
    // mutate the store, fire another event, play a sound... // no
});
```

## Note on `Clone` systems

Some systems are registered as both update and draw (via `.clone()`). Any mutable state shared between those two roles — the event queue, timers, etc. — must be behind `Rc<RefCell<...>>`, not plain fields. A plain `HashMap`/`VecDeque` field is copied by `#[derive(Clone)]`, leaving the update and draw copies with separate state.
