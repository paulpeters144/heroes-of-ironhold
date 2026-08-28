use macroquad::prelude::{Color, Texture2D, Vec2};

#[derive(Clone, Debug)]
pub struct StaticImage {
    pub source: Texture2D,
    pub position: Vec2,
    pub size: Vec2,
    pub tint: Color,
    pub flip_x: bool,
    pub visible: bool,
    pub z_idx: i32,
}
