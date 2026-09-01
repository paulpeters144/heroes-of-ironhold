# Description

Create an enemy factory that produces the "ram head" enemy from the new spritesheet `assets/images/enemies/anim-ram-head.png` (8 frames, 64x64 each). Spawn the enemy in the battle test scene, where it moves toward the player's knight and performs its club-swipe attack when in range. Follow the existing knight integration as the reference pattern.

# TODO

- [x] Register the enemy image asset
- [x] Add ram-head animation definitions (idle, walk, attack)
- [x] Create the enemy factory
- [x] Create enemy AI system (approach + attack)
- [x] Spawn the enemy in the battle test scene
- [x] Build and verify on desktop

# TODO Explanation

## Register the enemy image asset
Add an `Id` variant for the ram-head image in the image category module in `src/access/ids.rs` and register `enemies/anim-ram-head.png` in that module's manifest, following the pattern of existing image assets.

## Add ram-head animation definitions
Define the sprite animation clips for the ram-head enemy using the 64x64 frames: idle is a single-frame clip of frame 1, walking is frames 1-5 (frame 1 is intentionally part of the walk cycle as well as the idle frame), attack is frames 6-8. Mirror however the knight's animations are defined (sprite sheet slicing + animation config), storing any needed handles on the struct at construction time rather than in update/draw. Animation frame rate matches the knight's for now, kept as an easily tunable value since it will be adjusted for feel later.

## Create the enemy factory
Create a factory (mirroring the knight's factory) that builds a ram-head enemy entity with the components it needs: position/transform, sprite + animation, movement, and combat-related components. Extract all needed assets during construction (`new()`/`factory()`), never in `update()`/`draw()`.

## Create enemy AI system (approach + attack)
Add a `sys_*.rs` system (per project naming rules) that each frame: moves the enemy toward the knight using `ctx.dt`, plays the walk animation while moving, and when within attack range plays the attack animation (club swipe). Idle plays when neither moving nor attacking. Movement speed is a placeholder set noticeably slower than the knight's (roughly half, easy to tune after seeing it in game). The attack is animation-only for now — no damage is dealt to the knight and no health/damage handling is added. Attack range just determines when the enemy stops moving and starts swinging.

## Spawn the enemy in the battle test scene
Instantiate a single enemy via the new factory in the battle test scene's setup, positioned at the center of the tiled map.

## Build and verify on desktop
Run `cargo run -p heroes-of-ironhold-desktop` (or at least `cargo build` plus tests if applicable) and confirm the enemy spawns, walks toward the knight, idles, and attacks in range without errors or asset warnings.

# Open Questions

All questions resolved.

# Out of Scope

- Damage dealing, health, or death handling (attack is animation-only for now)
- Generalizing the factory to support multiple enemy types
- Pathfinding, obstacle avoidance, or multi-enemy coordination
- Knight AI changes (knight fighting back or reacting to the attack)
- Web client build/verification
