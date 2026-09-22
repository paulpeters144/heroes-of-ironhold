use macroquad::prelude::Vec2;

/// A circular collision body.
///
/// For dynamic entities (whose parent has an `Animation` child) the `center`
/// field is ignored — the collision system derives the center from the
/// `Animation` each frame. For static walls (spawned without an `Animation`
/// body) `center` is the authoritative position.
#[derive(Clone, Debug)]
pub struct CollisionCircle {
    pub radius: f32,
    pub center: Vec2,
}
