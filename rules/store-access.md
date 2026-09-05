# Store Access Rules

- Read entities and components from the store with the fluent accessor chain (`first`, `get_child`, ...). Do NOT write utility methods on the store, add new store methods, or mutate the store internals just to look something up. The store already exposes everything you need; traverse the hierarchy by chaining guards. Example:
  ```rust
  let current_frame = self
      .store
      .first::<PlayerOne>()
      .and_then(|player| self.store.get_child::<Knight>(&player))
      .and_then(|knight| self.store.get_child::<Animation>(&knight))
      .map(|a| a.current_frame);
  ```
  Key accessor methods (see `crates/pico-entity-store/src/store.rs`): `first::<T>()`/`first_mut::<T>()` find the first live component of type `T`; `get_child::<T>(&parent)`/`get_child_mut::<T>(&parent)` find the first direct child of `parent` of type `T`; `all::<T>()`/`all_mut::<T>()` iterate every component of type `T`; `get_by_id::<T>(id)`/`get_by_id_mut::<T>(id)` fetch by numeric id; `parent`/`children`/`descendants` navigate the hierarchy. Guards deref to the component (`&T`/`&mut T`) and expose `.id()` and `.entity_ref()`. Never modify the store for reads; mutate in place via a mutable guard instead.
