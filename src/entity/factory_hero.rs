use super::impact_frame::ImpactFrame;
use super::knight::{
    Effect, EffectKind, Facing, Shield, Sword, SLASH_LIFETIME, THRUST_LIFETIME,
};
use super::{Animation, CollisionRect, StaticImage};
use macroquad::prelude::{Color, Rect, Texture2D, Vec2};

pub const FRAME_SIZE: f32 = 64.0;
pub const FRAME_COUNT: usize = 6;
pub const SWORD_FRAME_SIZE: f32 = 32.0;
pub const SWORD_FRAME_COUNT: usize = 2;
pub const SHIELD_SIZE: f32 = 32.0;
pub const COLLISION_WIDTH_SCALE: f32 = 0.5;
pub const COLLISION_HEIGHT_SCALE: f32 = 0.35;

#[derive(Clone, Debug)]
pub struct KnightCfg {
    pub outfit: Texture2D,
    pub sword: Texture2D,
    pub shield: Texture2D,
    pub impact: Texture2D,
    pub thrust: Texture2D,
    pub swipe: Texture2D,
}

pub struct KnightParts {
    pub body: Animation,
    pub shield: Shield,
    pub shield_image: StaticImage,
    pub sword: Sword,
    pub sword_animation: Animation,
    pub facing: Facing,
    pub collision_rect: CollisionRect,
    pub impact_frame: ImpactFrame,
    pub impact_image: StaticImage,
    pub thrust: Effect,
    pub slash: Effect,
}

pub struct HeroFactory;

impl HeroFactory {
    pub fn create_knight(cfg: KnightCfg) -> KnightParts {
        let body = Animation {
            source: cfg.outfit,
            position: Vec2::ZERO,
            frame_width: FRAME_SIZE,
            frame_height: FRAME_SIZE,
            frame_count: FRAME_COUNT,
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

        let shield_image = StaticImage {
            source: cfg.shield,
            position: Vec2::ZERO,
            size: Vec2::new(SHIELD_SIZE, SHIELD_SIZE),
            scale: 1.0,
            tint: Color::new(1.0, 1.0, 1.0, 1.0),
            flip_x: false,
            visible: true,
            z_idx: 1.0,
        };

        let sword_animation = Animation {
            source: cfg.sword,
            position: Vec2::ZERO,
            frame_width: SWORD_FRAME_SIZE,
            frame_height: SWORD_FRAME_SIZE,
            frame_count: SWORD_FRAME_COUNT,
            current_frame: 0,
            running: false,
            frame_duration: 0.12,
            tint: Color::new(1.0, 1.0, 1.0, 1.0),
            flip_x: false,
            dest_size: Vec2::new(SWORD_FRAME_SIZE, SWORD_FRAME_SIZE),
            scale: 1.0,
            visible: true,
            z_idx: 2.0,
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

        let sword_rect = sword_animation.rect();
        let x = sword_rect.right() - sword_animation.position.x;
        let thrust_offset = Vec2::new(x, (sword_rect.h - cfg.thrust.height()) * 0.5);
        let slash_offset = Vec2::new(x + 15.0, (sword_rect.h - cfg.swipe.height()) * 0.5);

        let thrust = Effect {
            kind: EffectKind::Thrust,
            texture: cfg.thrust,
            offset: thrust_offset,
            age: 0.0,
            lifetime: THRUST_LIFETIME,
            visible: false,
        };
        let slash = Effect {
            kind: EffectKind::Slash,
            texture: cfg.swipe,
            offset: slash_offset,
            age: 0.0,
            lifetime: SLASH_LIFETIME,
            visible: false,
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
            impact_frame: ImpactFrame,
            impact_image,
            thrust,
            slash,
        }
    }
}
