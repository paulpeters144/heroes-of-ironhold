use crate::Drawable;
use macroquad::prelude::{draw_texture_ex, vec2, Color, DrawTextureParams, Rect, Texture2D, Vec2};

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
    pub scale: f32,
    pub visible: bool,
    pub z_idx: f32,
}

impl Animation {
    pub fn rect(&self) -> Rect {
        let size = if self.dest_size == Vec2::ZERO {
            vec2(self.frame_width, self.frame_height)
        } else {
            self.dest_size
        };
        Rect::new(
            self.position.x,
            self.position.y,
            size.x * self.scale,
            size.y * self.scale,
        )
    }
}

impl Drawable for Animation {
    fn draw(&self) {
        let source = Rect::new(
            self.current_frame as f32 * self.frame_width,
            0.0,
            self.frame_width,
            self.frame_height,
        );
        let dest_size = if self.dest_size == Vec2::ZERO {
            vec2(self.frame_width, self.frame_height)
        } else {
            self.dest_size
        } * self.scale;

        draw_texture_ex(
            &self.source,
            self.position.x,
            self.position.y,
            self.tint,
            DrawTextureParams {
                dest_size: Some(dest_size),
                source: Some(source),
                flip_x: self.flip_x,
                ..Default::default()
            },
        );
    }

    fn zdx(&self) -> f32 {
        self.z_idx
    }
}
