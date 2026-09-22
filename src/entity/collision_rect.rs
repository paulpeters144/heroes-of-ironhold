use macroquad::prelude::Rect;

/// A static axis-aligned rectangular collision boundary (e.g. a Tiled
/// "collisions" object). Spawned as a top-level entity with no `Animation`
/// body; the collision system treats it as immovable and pushes dynamic circles
/// out of it.
#[derive(Clone, Debug)]
pub struct CollisionRect {
    pub rect: Rect,
}
