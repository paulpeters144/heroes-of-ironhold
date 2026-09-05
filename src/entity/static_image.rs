use crate::Drawable;
use macroquad::prelude::{draw_texture_ex, Color, DrawTextureParams, Rect, Texture2D, Vec2};

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

impl Drawable for StaticImage {
    fn draw(&self) {
        draw_texture_ex(
            &self.source,
            self.position.x,
            self.position.y,
            self.tint,
            DrawTextureParams {
                dest_size: Some(self.size * self.scale),
                flip_x: self.flip_x,
                ..Default::default()
            },
        );
    }

    fn zdx(&self) -> i32 {
        self.z_idx
    }
}
