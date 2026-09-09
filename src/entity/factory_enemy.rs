use crate::entity::enemy::{RamHead, RAM_HEAD_FRAME_COUNT};
use crate::{images, Animation, Assets, CollisionRect};
use macroquad::prelude::{Color, Rect, Vec2};

pub const FRAME_SIZE: f32 = 64.0;
pub const COLLISION_WIDTH_SCALE: f32 = 0.5;
pub const COLLISION_HEIGHT_SCALE: f32 = 0.35;

pub struct RamHeadParts {
    pub marker: RamHead,
    pub body: Animation,
    pub collision_rect: CollisionRect,
}

pub struct EnemyFactory<'a> {
    assets: &'a Assets,
}

impl<'a> EnemyFactory<'a> {
    pub fn new(assets: &'a Assets) -> Self {
        Self { assets }
    }

    pub fn create_ram_head(&self) -> RamHeadParts {
        let body = Animation {
            source: self.assets.texture(images::Enemy::RamHead),
            position: Vec2::ZERO,
            frame_width: FRAME_SIZE,
            frame_height: FRAME_SIZE,
            frame_count: RAM_HEAD_FRAME_COUNT,
            current_frame: 0,
            running: false,
            tint: Color::new(1.0, 1.0, 1.0, 1.0),
            flip_x: false,
            dest_size: Vec2::new(FRAME_SIZE, FRAME_SIZE),
            scale: 1.0,
            visible: true,
            z_idx: 0.0,
        };

        RamHeadParts {
            marker: RamHead,
            body,
            collision_rect: CollisionRect {
                rect: Rect::new(
                    0.0,
                    0.0,
                    FRAME_SIZE * COLLISION_WIDTH_SCALE,
                    FRAME_SIZE * COLLISION_HEIGHT_SCALE,
                ),
            },
        }
    }
}
