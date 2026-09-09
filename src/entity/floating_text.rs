use crate::util::view_scale;
use crate::Drawable;
use macroquad::prelude::{
    draw_text_ex, measure_text, vec2, Color, Font, Rect, TextParams, Vec2, WHITE,
};
use macroquad::rand::gen_range;

// Logical view resolution, matching `Config::v_width`/`v_height`. The render
// pipeline blits this view to the window by `view_scale`, so rasterizing the
// text at that higher resolution (then drawing it back down) keeps it crisp.
const V_WIDTH: f32 = 640.0;
const V_HEIGHT: f32 = 360.0;

const ON_TOP_Z: f32 = 1000.0;
const LIFETIME: f32 = 0.8;
const RISE_OFFSET: f32 = 8.0;
const RISE_DISTANCE: f32 = 26.0;
const POP_FRACTION: f32 = 0.18;
const EXIT_FRACTION: f32 = 0.25;
const FADE_IN_FRACTION: f32 = 0.06;
const COLOR_SETTLE_FRACTION: f32 = 0.3;
const FLASH_TOWARD_WHITE: f32 = 0.55;

#[derive(Clone, Debug)]
pub struct FloatingText {
    pub text: String,
    pub position: Vec2,
    pub font: Font,
    pub font_size: u16,
    pub color: Color,
    pub age: f32,
    pub lifetime: f32,
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
            visible: true,
            z_idx: ON_TOP_Z,
        }
    }

    /// Punchy scale pop: grows from 0 with an overshoot past 1.0 and settles,
    /// holds, then shrinks out over the final `EXIT_FRACTION` of the lifetime.
    fn scale(&self, t: f32) -> f32 {
        let exit = 1.0 - EXIT_FRACTION;
        if t < POP_FRACTION {
            ease_out_back(t / POP_FRACTION)
        } else if t < exit {
            1.0
        } else {
            1.0 - ease_in_cubic((t - exit) / EXIT_FRACTION)
        }
    }

    /// Fades in almost instantly, holds, then fades out over the exit window.
    fn alpha(&self, t: f32) -> f32 {
        let fade_in = (t / FADE_IN_FRACTION).min(1.0);
        let exit = 1.0 - EXIT_FRACTION;
        let fade_out = if t > exit {
            1.0 - ease_in_cubic((t - exit) / EXIT_FRACTION)
        } else {
            1.0
        };
        fade_in * fade_out
    }

    /// Starts as a bright white flash and settles into the target color.
    fn color_at(&self, t: f32, alpha: f32) -> Color {
        let settle = (t / COLOR_SETTLE_FRACTION).min(1.0);
        let flash = lerp_color(self.color, WHITE, FLASH_TOWARD_WHITE);
        let mut color = lerp_color(flash, self.color, settle);
        color.a = alpha;
        color
    }
}

impl Drawable for FloatingText {
    fn draw(&self) {
        let t = (self.age / self.lifetime).clamp(0.0, 1.0);
        let anim = self.scale(t);
        let alpha = self.alpha(t);
        if alpha <= 0.0 || anim <= 0.0 {
            return;
        }

        // Rasterize the glyph at the blit (render-target → window) resolution,
        // then draw it back down. `font_size * font_scale` still equals
        // `nominal * anim`, so layout is unchanged — only the atlas is higher
        // resolution, which keeps the animated upscale crisp instead of fuzzy.
        let blit = view_scale::view_scale(V_WIDTH, V_HEIGHT).0;
        let blit = if blit > 0.0 { blit } else { 1.0 };
        let font_size = (self.font_size as f32 * blit).round().max(1.0) as u16;
        let font_scale = anim / blit;

        let color = self.color_at(t, alpha);
        let y = self.position.y - RISE_DISTANCE * ease_out_cubic(t);

        let dims = measure_text(&self.text, Some(&self.font), font_size, font_scale);
        let x = self.position.x - dims.width * 0.5;
        let baseline = y + dims.offset_y;

        let outline = Color::new(0.0, 0.0, 0.0, alpha);
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
                    font_size,
                    font_scale,
                    color: outline,
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
                font_size,
                font_scale,
                color,
                ..Default::default()
            },
        );
    }

    fn zdx(&self) -> f32 {
        self.z_idx
    }
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

fn ease_in_cubic(t: f32) -> f32 {
    t * t * t
}

/// Overshoots past 1.0 (peaking around 1.25) before settling back to 1.0.
fn ease_out_back(t: f32) -> f32 {
    const C1: f32 = 3.0;
    const C3: f32 = C1 + 1.0;
    let u = t - 1.0;
    1.0 + C3 * u * u * u + C1 * u * u
}
