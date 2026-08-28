use macroquad::prelude::Vec2;

pub(crate) const THRUST_LIFETIME: f32 = 0.16;
pub(crate) const SLASH_LIFETIME: f32 = 0.16;
pub(crate) const GLOW_IN_FRACTION: f32 = 0.2;
pub(crate) const HOLD_START: f32 = 0.5;
pub(crate) const THRUST_OFFSET_X: f32 = 4.0;
pub(crate) const SLASH_OFFSET_X: f32 = 35.0;
pub(crate) const SLASH_OFFSET_Y: f32 = -5.0;

#[derive(Clone, Copy, Debug)]
pub enum EffectKind {
    Thrust,
    Slash,
}

#[derive(Clone, Debug)]
pub struct Effect {
    pub kind: EffectKind,
    pub age: f32,
    pub lifetime: f32,
    pub origin: Vec2,
}
