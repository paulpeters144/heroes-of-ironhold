use macroquad::prelude::{Texture2D, Vec2};

#[derive(Clone)]
pub struct Animation {
    pub texture: Texture2D,
    pub frame: usize,
    pub frame_count: usize,
    pub frame_time: f32,
    pub elapsed: f32,
    pub pos: Vec2,
    pub scale: f32,
}
