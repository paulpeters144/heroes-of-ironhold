use macroquad::prelude::*;

pub struct Style {
    pub fill: Color,
    pub border: Color,
    pub border_width: f32,
    pub radius: f32,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            fill: WHITE,
            border: BLACK,
            border_width: 1.0,
            radius: 0.0,
        }
    }
}

impl Style {
    pub fn rounded(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }
}
