# Store Access Rules

- Read entities and components from the store with the fluent accessor chain (`first`, `get_child`, ...) and inline the chain at the call site. Do NOT wrap a single store lookup in a private helper method, and do NOT write utility methods on the store, add new store methods, or mutate the store internals just to look something up. The store already exposes everything you need; traverse the hierarchy by chaining guards.

## Do: inline the fluent chain where you need it

```rust
fn update(&mut self, ctx: &mut Context) {
    let Some(knight_ref) = self
        .store
        .first::<PlayerOne>()
        .and_then(|player| self.store.get_child::<Knight>(&player))
        .map(|k| k.entity_ref())
    else {
        return;
    };
    // ...
}
```

A longer traversal is fine too — keep chaining `and_then` / `map`:

```rust
let current_frame = self
    .store
    .first::<PlayerOne>()
    .and_then(|player| self.store.get_child::<Knight>(&player))
    .and_then(|knight| self.store.get_child::<Animation>(&knight))
    .map(|a| a.current_frame);
```

## Not: a private method that only wraps one store lookup

```rust
// Don't add this to a system struct.
fn knight_ref(&self) -> Option<EntityRef> {
    let player = self.store.first::<PlayerOne>()?;
    self.store
        .get_child::<Knight>(&player)
        .map(|k| k.entity_ref())
}
```

If the exact same chain is needed at several call sites, that is a sign the lookup itself may be worth a real component relationship or shared constant — but do not paper over it with a single-purpose getter that hides the traversal.

## Not: utility methods on the store itself

```rust
// Don't extend EStore (or the store crate) with lookup helpers.
impl EStore {
    fn knight_ref(&self) -> Option<EntityRef> {
        // ...
    }
}
```

The store is a dumb container. Reads belong in the calling system, written as fluent chains; mutations belong in place via a mutable guard:

```rust
// Mutate in place through a guard — never via a store helper.
if let Some(anim_ref) = self
    .store
    .get_by_id::<Knight>(knight_ref.id())
    .and_then(|k| self.store.get_child::<Animation>(&k))
    .map(|a| a.entity_ref())
{
    self.store.update::<Animation, _>(&anim_ref, |a| a.flip_x = mirror);
}
```

## Accessor reference

Key accessor methods (see `crates/pico-entity-store/src/store.rs`):

- `first::<T>()` / `first_mut::<T>()` — find the first live component of type `T`.
- `get_child::<T>(&parent)` / `get_child_mut::<T>(&parent)` — find the first direct child of `parent` of type `T`.
- `all::<T>()` / `all_mut::<T>()` — iterate every component of type `T`.
- `get_by_id::<T>(id)` / `get_by_id_mut::<T>(id)` — fetch by numeric id.
- `parent` / `children` / `descendants` — navigate the hierarchy.

Guards deref to the component (`&T` / `&mut T`) and expose `.id()` and `.entity_ref()`.
