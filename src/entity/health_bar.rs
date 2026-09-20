use super::Drawable;
use macroquad::prelude::{
    draw_texture_ex, vec2, Color, DrawTextureParams, FilterMode, Image, Rect, Texture2D, Vec2,
    WHITE,
};

const BORDER: f32 = 1.0;
const TRACK_RADIUS: f32 = 3.0;
const FILL_RADIUS: f32 = 2.0;

const BORDER_COLOR: Color = Color::new(0.10, 0.10, 0.12, 1.0);
const BACKGROUND_COLOR: Color = Color::new(0.16, 0.16, 0.18, 0.95);

fn health_color(percent: f32) -> Color {
    let p = percent.clamp(0.0, 1.0);
    let (r, g) = if p > 0.5 {
        let t = (1.0 - p) * 2.0;
        (t, 1.0)
    } else {
        (1.0, p * 2.0)
    };
    Color::new(r, g, 0.08, 1.0)
}

fn sd_round_box(px: f32, py: f32, hx: f32, hy: f32, radius: f32) -> f32 {
    let qx = px.abs() - (hx - radius);
    let qy = py.abs() - (hy - radius);
    let ax = qx.max(0.0);
    let ay = qy.max(0.0);
    (ax * ax + ay * ay).sqrt() + qx.max(qy).min(0.0) - radius
}

fn make_texture(img: Image) -> Texture2D {
    let texture = Texture2D::from_image(&img);
    texture.set_filter(FilterMode::Nearest);
    texture
}

fn track_texture(width: u16, height: u16) -> Texture2D {
    let mut img = Image::gen_image_color(width, height, Color::new(0.0, 0.0, 0.0, 0.0));
    let hx = width as f32 * 0.5;
    let hy = height as f32 * 0.5;
    for y in 0..height {
        for x in 0..width {
            let px = x as f32 + 0.5 - hx;
            let py = y as f32 + 0.5 - hy;
            if sd_round_box(px, py, hx, hy, TRACK_RADIUS) > 0.0 {
                continue;
            }
            let inner = sd_round_box(px, py, hx - BORDER, hy - BORDER, TRACK_RADIUS - BORDER);
            let color = if inner <= 0.0 {
                BACKGROUND_COLOR
            } else {
                BORDER_COLOR
            };
            img.set_pixel(x as u32, y as u32, color);
        }
    }
    make_texture(img)
}

fn fill_texture(width: u16, height: u16) -> Texture2D {
    let mut img = Image::gen_image_color(width, height, Color::new(0.0, 0.0, 0.0, 0.0));
    let hx = width as f32 * 0.5;
    let hy = height as f32 * 0.5;
    for y in 0..height {
        for x in 0..width {
            let px = x as f32 + 0.5 - hx;
            let py = y as f32 + 0.5 - hy;
            if sd_round_box(px, py, hx, hy, FILL_RADIUS) > 0.0 {
                continue;
            }
            let t = (y as f32 + 0.5) / height as f32;
            let lum = 1.0 - 0.35 * t;
            img.set_pixel(x as u32, y as u32, Color::new(lum, lum, lum, 1.0));
        }
    }
    make_texture(img)
}

/// Presentation-only health bar drawn above an entity. The bar is a child of
/// the entity it represents; the update system positions it from the parent's
/// `Animation` and fills it according to `percent` (0.0..=1.0).
#[derive(Clone, Debug)]
pub struct HealthBar {
    pub width: f32,
    pub height: f32,
    pub percent: f32,
    pub visible: bool,
    pub z_idx: f32,
    pub position: Vec2,
    pub track: Texture2D,
    pub fill: Texture2D,
}

impl HealthBar {
    pub fn new(width: f32, height: f32) -> Self {
        let track = track_texture(width as u16, height as u16);
        let fill = fill_texture(
            (width - BORDER * 2.0) as u16,
            (height - BORDER * 2.0) as u16,
        );
        Self {
            width,
            height,
            percent: 1.0,
            visible: true,
            z_idx: 0.0,
            position: Vec2::ZERO,
            track,
            fill,
        }
    }
}

impl Default for HealthBar {
    fn default() -> Self {
        Self::new(40.0, 6.0)
    }
}

impl Drawable for HealthBar {
    fn draw(&self) {
        draw_texture_ex(
            &self.track,
            self.position.x,
            self.position.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(self.width, self.height)),
                ..Default::default()
            },
        );

        let inner_w = self.width - BORDER * 2.0;
        let inner_h = self.height - BORDER * 2.0;
        let fill_w = inner_w * self.percent.clamp(0.0, 1.0);
        if fill_w > 0.0 {
            draw_texture_ex(
                &self.fill,
                self.position.x + BORDER,
                self.position.y + BORDER,
                health_color(self.percent),
                DrawTextureParams {
                    dest_size: Some(vec2(fill_w, inner_h)),
                    source: Some(Rect::new(0.0, 0.0, fill_w, inner_h)),
                    ..Default::default()
                },
            );
        }
    }

    fn zdx(&self) -> f32 {
        self.z_idx
    }
}
