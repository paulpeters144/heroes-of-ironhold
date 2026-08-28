use macroquad::prelude::{Color, Texture2D, Vec2};

#[derive(Clone, Debug)]
pub struct Animation {
    pub source: Texture2D,
    pub position: Vec2,
    pub frame_width: f32,
    pub frame_height: f32,
    pub frame_count: usize,
    pub current_frame: usize,
    pub running: bool,
    pub tint: Color,
    pub flip_x: bool,
    pub dest_size: Vec2,
    pub visible: bool,
    pub z_idx: i32,
}
