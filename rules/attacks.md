# Attack Rect Rules

- Anything that deals damage (melee swings, projectile blades, enemy charges, spells) MUST own an `AttackRect` component. The `AttackRect` is the single source of truth for where an attack can hit: `rects` holds the active hit rectangles and `visible` says whether an attack is currently in flight.
- Attach the `AttackRect` as a direct child of the entity that owns the attack (e.g. the knight's `Sword`, a `RamHead`, each flying sword's `Animation`).
- Spawn it inert: `rects: Vec::new()` and `visible: false`.
- Set `rects` and `visible = true` the frame the attack is live, and set `visible = false` when the attack ends.
- Do all hit detection against `area.rects`, gated by `area.visible`. Never hand-roll a hit zone that bypasses the `AttackRect` component.

## Do: give every attacker an AttackRect and hit-test through it

```rust
// Spawn the attacker with an inert AttackRect.
let area = AttackRect {
    rects: Vec::new(),
    visible: false,
}
.into_child();
let anim_ref = store.add(anim, &[area]).expect("spawn attacker");

// Each frame the blade is live, move the rect with it.
store.update::<AttackRect, _>(&area_ref, |area| {
    area.rects = vec![blade_rect];
    area.visible = true;
});

// Damage only via the rect.
if area.visible && area.rects.iter().any(|r| did_attack(*r, &data)) {
    // ... apply damage ...
}
```

## Not: ad-hoc hit zones that bypass the component

```rust
// Don't damage enemies with a rect that lives nowhere in the store.
if enemy_rect.overlaps(&blade) {
    // ... apply damage ... // no
}
```

If the attack never uses its `AttackRect`, remove the component instead of leaving a stale, unused one attached.