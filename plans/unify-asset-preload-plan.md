# Description
Let `Assets::preload` accept a single heterogeneous slice of any asset ID (e.g. `images::Knight::Knight1`, `file::File::TestTmx`, `images::TilesetImage::HoiBg` all in one `&[...]`) with no `.into()` or wrapping — IDs are referenced directly. Implementation is confined to the assets access layer (`src/access/`); the existing ID category modules are left untouched.

# TODO
- [ ] Drop the `Copy` supertrait from `AssetId` so it becomes object-safe
- [ ] Change `preload` to take `&[&dyn AssetId]`
- [ ] Update the two call sites to a single `preload` array

# TODO Explanation

## Drop the `Copy` supertrait from `AssetId`
`preload<T: AssetId>(&mut self, ids: &[T])` is monomorphic: one concrete `T` per slice. Mixing `Knight`, `File`, and `TilesetImage` in a single literal requires a common type. A `dyn AssetId` trait object is the only way to reference IDs directly (no `.into()`, no wrapper enum). The current trait is `pub trait AssetId: Copy`, and `Copy` is not object-safe, so `dyn AssetId` is impossible today.

Change `src/access/ids.rs` line 11 from `pub trait AssetId: Copy` to `pub trait AssetId`.

Safety: `Copy` is not relied on anywhere in the access layer. The typed accessors (`texture`, `image`, `font`, `sound`, `file`, `shader`) take `T: AssetId` by value and call `id.path()` once, which consumes the value — no copy needed. `preload` iterates references. All concrete ID enums still derive `Clone, Copy`, so passing them by value elsewhere keeps working.

## Change `preload` to take `&[&dyn AssetId]`
In `src/access/assets.rs`, change the signature:

```rust
pub async fn preload(&mut self, ids: &[&dyn AssetId]) {
    for id in ids {
        let path = id.path();
        match id.kind() {
            // ... existing arms unchanged ...
        }
    }
}
```

`id` is `&&dyn AssetId`; `id.path()` / `id.kind()` resolve via auto-deref. No body changes beyond the signature.

## Update the two call sites to a single `preload` array
In `src/scene/battle_test/scene.rs` (currently lines 155-179) collapse the three `preload` calls into one, referencing IDs directly with a leading `&`:

```rust
self.assets
    .preload(&[
        &images::Knight::Knight1,
        &images::Knight::Shield1,
        &images::Knight::Sword1,
        &images::Knight::Knight2,
        &images::Knight::Shield2,
        &images::Knight::Sword2,
        &images::Knight::Knight3,
        &images::Knight::Shield3,
        &images::Knight::Sword3,
        &images::Knight::ThrustGraphic,
        &images::Knight::SwipeGraphic,
        &file::File::TestTmx,
        &file::File::HoiBgTsx,
        &file::File::HoiCharsTsx,
        &images::TilesetImage::HoiBg,
        &images::TilesetImage::HoiChars,
    ])
    .await;
```

Do the same for `src/scene/asset_preview/scene.rs` (around line 229), keeping it a single array as well. (Confirmed: yes, collapse it too.)

# Open Questions
- The slice elements need a leading `&` (`&images::Knight::Knight1`) because Rust slices are homogeneous and `&dyn AssetId` is the common type. Is `&` per element acceptable, or do you want a zero-prefix form (which would require merging all ID enums into one flattened enum, changing the category modules)?
- Naming: keep the existing `images`/`file`/etc. module paths as-is, or introduce a new `assets::Images::...` namespace? You wrote `assets::Images::Knight::Knight1` — please confirm the exact path you want.

# Out of Scope
- No changes to the individual ID category modules (`texture`, `font`, `sound`, `file`, `shader`, `images`).
- No changes to `pico_entity_store` or `src/util/estore.rs`.
- No change to `preload`'s loading logic beyond the access-layer signature change.
