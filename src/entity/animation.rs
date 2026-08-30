use macroquad::prelude::{vec2, Color, Rect, Texture2D, Vec2};

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
    pub z_idx: i32,
}

impl Animation {
    pub fn rect(&self) -> Rect {
        let size = if self.dest_size == Vec2::ZERO {
            vec2(self.frame_width, self.frame_height)
        } else {
            self.dest_size
        };
        Rect::new(self.position.x, self.position.y, size.x * self.scale, size.y * self.scale)
    }
}
