use crate::Drawable;
use macroquad::prelude::{
    draw_text_ex, measure_text, vec2, Color, Font, Rect, TextParams, Vec2,
};
use macroquad::rand::gen_range;

const ON_TOP_Z: f32 = 1000.0;
const LIFETIME: f32 = 0.6;
const RISE_SPEED: f32 = 24.0;
const RISE_OFFSET: f32 = 8.0;
const SCALE_GROW_FRACTION: f32 = 0.25;
const SCALE_START: f32 = 0.5;
const SCALE_PEAK: f32 = 1.0;

#[derive(Clone, Debug)]
pub struct FloatingText {
    pub text: String,
    pub position: Vec2,
    pub font: Font,
    pub font_size: u16,
    pub color: Color,
    pub age: f32,
    pub lifetime: f32,
    pub rise_speed: f32,
    pub visible: bool,
    pub z_idx: f32,
}

impl FloatingText {
    /// Builds a new `FloatingText` anchored above `rect` at a random x within the rect's span.
    pub fn new(text: String, rect: Rect, font: Font, font_size: u16, color: Color) -> Self {
        let x = gen_range(rect.x, rect.x + rect.w);
        Self {
            text,
            position: vec2(x, rect.y - RISE_OFFSET),
            font,
            font_size,
            color,
            age: 0.0,
            lifetime: LIFETIME,
            rise_speed: RISE_SPEED,
            visible: true,
            z_idx: ON_TOP_Z,
        }
    }

    /// Current scale factor: lerps from `SCALE_START` up to `SCALE_PEAK` over
    /// the first `SCALE_GROW_FRACTION` of the lifetime, then back down to `SCALE_START`.
    fn scale(&self) -> f32 {
        let grow = self.lifetime * SCALE_GROW_FRACTION;
        let t = if self.age < grow {
            self.age / grow
        } else {
            1.0 - (self.age - grow) / (self.lifetime - grow).max(f32::EPSILON)
        };
        SCALE_START + (SCALE_PEAK - SCALE_START) * t
    }
}

impl Drawable for FloatingText {
    fn draw(&self) {
        let scale = self.scale();
        let dims = measure_text(&self.text, Some(&self.font), self.font_size, 1.0);
        let x = self.position.x - dims.width * 0.5 * scale;
        let baseline = self.position.y + dims.offset_y * scale;

        for (dx, dy) in [
            (-1.0, 0.0),
            (1.0, 0.0),
            (0.0, -1.0),
            (0.0, 1.0),
            (-1.0, -1.0),
            (1.0, -1.0),
            (-1.0, 1.0),
            (1.0, 1.0),
        ] {
            draw_text_ex(
                &self.text,
                x + dx,
                baseline + dy,
                TextParams {
                    font: Some(&self.font),
                    font_size: self.font_size,
                    font_scale: scale,
                    color: Color::new(0.0, 0.0, 0.0, 1.0),
                    ..Default::default()
                },
            );
        }

        draw_text_ex(
            &self.text,
            x,
            baseline,
            TextParams {
                font: Some(&self.font),
                font_size: self.font_size,
                font_scale: scale,
                color: self.color,
                ..Default::default()
            },
        );
    }

    fn zdx(&self) -> f32 {
        self.z_idx
    }
}
