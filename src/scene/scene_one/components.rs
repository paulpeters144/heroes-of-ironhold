use crate::image;
use macroquad::prelude::{Rect, Texture2D, Vec2};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Anim {
    Idle,
    Run,
    Jump,
    Hurt,
}

#[derive(Clone)]
pub struct Player {
    pub pos: Vec2,
    pub vel: Vec2,
    pub facing: f32,
    pub grounded: bool,
    pub anim: Anim,
    pub hurt: bool,
}

#[derive(Clone)]
pub struct Physics {
    pub move_speed: f32,
    pub gravity: f32,
    pub jump_velocity: f32,
    pub sprite_size: f32,
    pub hitbox_w: f32,
    pub hitbox_h: f32,
}

impl Physics {
    pub fn hitbox_off_x(&self) -> f32 {
        (self.sprite_size - self.hitbox_w) * 0.5
    }

    pub fn hitbox_off_y(&self) -> f32 {
        self.sprite_size - self.hitbox_h
    }

    pub fn hitbox(&self, pos: Vec2) -> Rect {
        Rect::new(
            pos.x + self.hitbox_off_x(),
            pos.y + self.hitbox_off_y(),
            self.hitbox_w,
            self.hitbox_h,
        )
    }

    pub fn hurtbox(&self, pos: Vec2) -> Rect {
        let w = self.hitbox_w * 0.5;
        let h = self.hitbox_h * 0.35;
        Rect::new(
            pos.x + (self.sprite_size - w) * 0.5,
            pos.y + (self.sprite_size - h) * 0.5,
            w,
            h,
        )
    }
}

#[derive(Clone)]
pub struct Sprite {
    pub texture: Texture2D,
    pub first_frame: usize,
    pub frame_w: f32,
    pub frame_h: f32,
}

#[derive(Clone)]
pub struct SpriteDef {
    pub image: image::Id,
    pub first_frame: usize,
    pub frame_count: usize,
    pub sheet_frames: usize,
    pub frame_time: f32,
    pub once: bool,
}

impl Sprite {
    pub fn from_def(def: &SpriteDef, assets: &crate::Assets) -> Sprite {
        let texture = assets
            .texture_from_image(def.image)
            .unwrap_or_else(Texture2D::empty);
        let frame_w = texture.width() / def.sheet_frames as f32;
        let frame_h = texture.height();
        Sprite {
            texture,
            first_frame: def.first_frame,
            frame_w,
            frame_h,
        }
    }
}

#[derive(Clone)]
pub struct Sprites {
    pub idle: Sprite,
    pub run: Sprite,
    pub jump: Sprite,
    pub hurt: Sprite,
}

impl Sprites {
    pub fn for_anim(&self, anim: Anim) -> &Sprite {
        match anim {
            Anim::Idle => &self.idle,
            Anim::Run => &self.run,
            Anim::Jump => &self.jump,
            Anim::Hurt => &self.hurt,
        }
    }
}

#[derive(Clone)]
pub struct PlayerSpriteDefs {
    pub idle: SpriteDef,
    pub run: SpriteDef,
    pub jump: SpriteDef,
    pub hurt: SpriteDef,
}

impl PlayerSpriteDefs {
    pub fn for_anim(&self, anim: Anim) -> &SpriteDef {
        match anim {
            Anim::Idle => &self.idle,
            Anim::Run => &self.run,
            Anim::Jump => &self.jump,
            Anim::Hurt => &self.hurt,
        }
    }
}

#[derive(Clone)]
pub struct AnimFrame {
    pub frame: usize,
    pub elapsed: f32,
    pub idle_delay: f32,
    pub done: bool,
}

#[derive(Clone)]
pub struct Blink {
    pub active: bool,
    pub timer: f32,
    pub blinks_left: u32,
    pub visible: bool,
    pub blink_count: u32,
    pub blink_interval: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EnemyAnim {
    Idle,
    Walk,
    Fall,
}

#[derive(Clone)]
pub struct Enemy {
    pub pos: Vec2,
    pub vel: Vec2,
    pub facing: f32,
    pub grounded: bool,
    pub anim: EnemyAnim,
}

#[derive(Clone)]
pub struct EnemySprites {
    pub idle: Sprite,
    pub walk: Sprite,
    pub fall: Sprite,
}

impl EnemySprites {
    pub fn for_anim(&self, anim: EnemyAnim) -> &Sprite {
        match anim {
            EnemyAnim::Idle => &self.idle,
            EnemyAnim::Walk => &self.walk,
            EnemyAnim::Fall => &self.fall,
        }
    }
}

#[derive(Clone)]
pub struct EnemySpriteDefs {
    pub idle: SpriteDef,
    pub walk: SpriteDef,
    pub fall: SpriteDef,
}

impl EnemySpriteDefs {
    pub fn for_anim(&self, anim: EnemyAnim) -> &SpriteDef {
        match anim {
            EnemyAnim::Idle => &self.idle,
            EnemyAnim::Walk => &self.walk,
            EnemyAnim::Fall => &self.fall,
        }
    }
}

#[derive(Clone)]
pub struct Patrol {
    pub a: f32,
    pub b: f32,
    pub target_x: f32,
    pub idle_timer: f32,
    pub reach_dist: f32,
    pub idle_time: f32,
}
