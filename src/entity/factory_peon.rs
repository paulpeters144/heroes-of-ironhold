use super::collision_circle::{CollisionCircle, CollisionGroup};
use super::impact_frame::ImpactFrame;
use super::peon::{Peon, PEON_FRAME_COUNT};
use super::{Animation, StaticImage};
use macroquad::prelude::{Color, Texture2D, Vec2};

pub const PEON_FRAME_SIZE: f32 = 64.0;
pub const PEON_COLLISION_RADIUS_SCALE: f32 = 0.095;

pub struct PeonParts {
    pub marker: Peon,
    pub body: Animation,
    pub collision_circle: CollisionCircle,
    pub impact_frame: ImpactFrame,
    pub impact_image: StaticImage,
}

#[derive(Clone, Debug)]
pub struct PeonCfg {
    pub body: Texture2D,
    pub impact: Texture2D,
}

pub struct PeonFactory;

impl PeonFactory {
    pub fn create_peon(cfg: PeonCfg) -> PeonParts {
        let body = Animation {
            source: cfg.body,
            position: Vec2::ZERO,
            frame_width: PEON_FRAME_SIZE,
            frame_height: PEON_FRAME_SIZE,
            frame_count: PEON_FRAME_COUNT,
            current_frame: 0,
            running: false,
            frame_duration: 0.12,
            tint: Color::new(1.0, 1.0, 1.0, 1.0),
            flip_x: false,
            dest_size: Vec2::new(PEON_FRAME_SIZE, PEON_FRAME_SIZE),
            scale: 1.0,
            visible: true,
            z_idx: 0.0,
        };

        let impact_image = StaticImage {
            source: cfg.impact,
            position: Vec2::ZERO,
            size: Vec2::new(PEON_FRAME_SIZE, PEON_FRAME_SIZE),
            scale: 1.0,
            tint: Color::new(1.0, 1.0, 1.0, 1.0),
            flip_x: false,
            visible: false,
            z_idx: 3.0,
        };

        PeonParts {
            marker: Peon,
            body,
            collision_circle: CollisionCircle {
                radius: PEON_FRAME_SIZE * PEON_COLLISION_RADIUS_SCALE,
                center: Vec2::ZERO,
                group: CollisionGroup::Peon,
            },
            impact_frame: ImpactFrame,
            impact_image,
        }
    }
}
