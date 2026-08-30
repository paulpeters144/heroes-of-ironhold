use macroquad::prelude::{Texture2D, Vec2};

pub(crate) const THRUST_LIFETIME: f32 = 0.16;
pub(crate) const SLASH_LIFETIME: f32 = 0.16;
pub(crate) const GLOW_IN_FRACTION: f32 = 0.2;
pub(crate) const HOLD_START: f32 = 0.5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectKind {
    Thrust,
    Slash,
}

#[derive(Clone, Debug)]
pub struct Effect {
    pub kind: EffectKind,
    pub texture: Texture2D,
    pub offset: Vec2,
    pub age: f32,
    pub lifetime: f32,
    pub visible: bool,
}
