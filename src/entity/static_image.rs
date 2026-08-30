use macroquad::prelude::{Color, Rect, Texture2D, Vec2};

#[derive(Clone, Debug)]
pub struct StaticImage {
    pub source: Texture2D,
    pub position: Vec2,
    pub size: Vec2,
    pub scale: f32,
    pub tint: Color,
    pub flip_x: bool,
    pub visible: bool,
    pub z_idx: i32,
}

impl StaticImage {
    pub fn rect(&self) -> Rect {
        Rect::new(
            self.position.x,
            self.position.y,
            self.size.x * self.scale,
            self.size.y * self.scale,
        )
    }
}
