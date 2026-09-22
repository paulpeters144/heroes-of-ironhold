use super::collision_circle::CollisionCircle;
use macroquad::prelude::Vec2;

#[derive(Clone, Debug)]
pub struct WallCfg {
    pub center: Vec2,
    pub radius: f32,
}

pub struct WallFactory;

impl WallFactory {
    /// Builds a static wall collider. Spawn it as a top-level entity (no
    /// parent / no `Animation` body); the collision system treats it as
    /// immovable and resolves dynamic circles against it.
    pub fn create(cfg: WallCfg) -> CollisionCircle {
        CollisionCircle {
            radius: cfg.radius,
            center: cfg.center,
        }
    }
}
