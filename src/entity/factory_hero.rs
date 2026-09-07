use crate::entity::knight::{Facing, Shield, Sword};
use crate::{images, Animation, Assets, CollisionRect, StaticImage};
use macroquad::prelude::{Color, Rect, Vec2};

pub const FRAME_SIZE: f32 = 64.0;
pub const FRAME_COUNT: usize = 6;
pub const SWORD_FRAME_SIZE: f32 = 32.0;
pub const SWORD_FRAME_COUNT: usize = 2;
pub const SHIELD_SIZE: f32 = 32.0;
pub const COLLISION_WIDTH_SCALE: f32 = 0.25;
pub const COLLISION_HEIGHT_SCALE: f32 = 0.5;

#[derive(Clone, Copy, Debug)]
pub struct KnightCfg {
    pub outfit: images::Knight,
    pub sword: images::Knight,
    pub shield: images::Knight,
}

pub struct KnightParts {
    pub body: Animation,
    pub shield: Shield,
    pub shield_image: StaticImage,
    pub sword: Sword,
    pub sword_animation: Animation,
    pub facing: Facing,
    pub collision_rect: CollisionRect,
}

pub struct HeroFactory<'a> {
    assets: &'a Assets,
}

impl<'a> HeroFactory<'a> {
    pub fn new(assets: &'a Assets) -> Self {
        Self { assets }
    }

    pub fn create_knight(&self, cfg: KnightCfg) -> KnightParts {
        let body = Animation {
            source: self.assets.texture(cfg.outfit),
            position: Vec2::ZERO,
            frame_width: FRAME_SIZE,
            frame_height: FRAME_SIZE,
            frame_count: FRAME_COUNT,
            current_frame: 0,
            running: false,
            tint: Color::new(1.0, 1.0, 1.0, 1.0),
            flip_x: false,
            dest_size: Vec2::new(FRAME_SIZE, FRAME_SIZE),
            scale: 1.0,
            visible: true,
            z_idx: 0.0,
        };

        let shield_image = StaticImage {
            source: self.assets.texture(cfg.shield),
            position: Vec2::ZERO,
            size: Vec2::new(SHIELD_SIZE, SHIELD_SIZE),
            scale: 1.0,
            tint: Color::new(1.0, 1.0, 1.0, 1.0),
            flip_x: false,
            visible: true,
            z_idx: 1.0,
        };

        let sword_animation = Animation {
            source: self.assets.texture(cfg.sword),
            position: Vec2::ZERO,
            frame_width: SWORD_FRAME_SIZE,
            frame_height: SWORD_FRAME_SIZE,
            frame_count: SWORD_FRAME_COUNT,
            current_frame: 0,
            running: false,
            tint: Color::new(1.0, 1.0, 1.0, 1.0),
            flip_x: false,
            dest_size: Vec2::new(SWORD_FRAME_SIZE, SWORD_FRAME_SIZE),
            scale: 1.0,
            visible: true,
            z_idx: 2.0,
        };

        KnightParts {
            body,
            shield: Shield,
            shield_image,
            sword: Sword,
            sword_animation,
            facing: Facing::Right,
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
