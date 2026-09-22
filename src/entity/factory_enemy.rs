use super::enemy::{RamHead, RAM_HEAD_FRAME_COUNT};
use super::impact_frame::ImpactFrame;
use super::{Animation, CollisionCircle, StaticImage};
use macroquad::prelude::{Color, Texture2D, Vec2};

pub const FRAME_SIZE: f32 = 64.0;
pub const COLLISION_RADIUS_SCALE: f32 = 0.095;

pub struct RamHeadParts {
    pub marker: RamHead,
    pub body: Animation,
    pub collision_circle: CollisionCircle,
    pub impact_frame: ImpactFrame,
    pub impact_image: StaticImage,
}

#[derive(Clone, Debug)]
pub struct RamHeadCfg {
    pub body: Texture2D,
    pub impact: Texture2D,
}

pub struct EnemyFactory;

impl EnemyFactory {
    pub fn create_ram_head(cfg: RamHeadCfg) -> RamHeadParts {
        let body = Animation {
            source: cfg.body,
            position: Vec2::ZERO,
            frame_width: FRAME_SIZE,
            frame_height: FRAME_SIZE,
            frame_count: RAM_HEAD_FRAME_COUNT,
            current_frame: 0,
            running: false,
            frame_duration: 0.12,
            tint: Color::new(1.0, 1.0, 1.0, 1.0),
            flip_x: false,
            dest_size: Vec2::new(FRAME_SIZE, FRAME_SIZE),
            scale: 1.0,
            visible: true,
            z_idx: 0.0,
        };

        let impact_image = StaticImage {
            source: cfg.impact,
            position: Vec2::ZERO,
            size: Vec2::new(FRAME_SIZE, FRAME_SIZE),
            scale: 1.0,
            tint: Color::new(1.0, 1.0, 1.0, 1.0),
            flip_x: false,
            visible: false,
            z_idx: 3.0,
        };

        RamHeadParts {
            marker: RamHead,
            body,
            collision_circle: CollisionCircle {
                radius: FRAME_SIZE * COLLISION_RADIUS_SCALE,
                center: Vec2::ZERO,
            },
            impact_frame: ImpactFrame,
            impact_image,
        }
    }
}
