use crate::entity::knight::{HeroStats, Knight};
use crate::{images, Assets, Config, Context, EStore, FontTag, GameFont, TextStyle};
use macroquad::prelude::*;
use std::rc::Rc;

// Layout (virtual 640x360 screen space): rounded portrait plate, pill-shaped
// HP/MP bars to its right, and a thin XP line with the level number below them.
const MARGIN: f32 = 8.0;
const PORTRAIT_SIZE: f32 = 50.0;
const FACE_SIZE: f32 = 40.0;
const PORTRAIT_RADIUS: f32 = 5.0;

const BAR_X: f32 = MARGIN + PORTRAIT_SIZE + 6.0;
const BAR_W: f32 = 126.0;
const BAR_H: f32 = 13.0;
const BAR_GAP: f32 = 4.0;
const HP_Y: f32 = MARGIN;
const MP_Y: f32 = HP_Y + BAR_H + BAR_GAP;
const XP_H: f32 = 5.0;
// XP sits lower: its medallion is anchored to the portrait's bottom edge.
const XP_CENTER_Y: f32 = MARGIN + PORTRAIT_SIZE - 9.0;
const BAR_RADIUS: f32 = 6.0;
const MEDAL_R: f32 = 9.0;

// Palette: dark steel frame, warm steel rim, deep navy track/back.
const FRAME: Color = Color::new(0.08, 0.09, 0.12, 1.0);
const RIM: Color = Color::new(0.45, 0.48, 0.56, 1.0);
const TRACK: Color = Color::new(0.055, 0.07, 0.17, 1.0);
const INK: Color = Color::new(0.02, 0.02, 0.04, 1.0);

// Fill gradients; the light entries are also the gloss color.
const HP_LIGHT: Color = Color::new(0.97, 0.37, 0.31, 1.0);
const HP_DARK: Color = Color::new(0.55, 0.13, 0.12, 1.0);
const MP_LIGHT: Color = Color::new(0.38, 0.58, 0.97, 1.0);
const MP_DARK: Color = Color::new(0.15, 0.28, 0.66, 1.0);
const XP_LIGHT: Color = Color::new(0.99, 0.81, 0.35, 1.0);
const XP_DARK: Color = Color::new(0.75, 0.52, 0.12, 1.0);

#[derive(Clone, Copy)]
struct FillStyle {
    light: Color,
    dark: Color,
}

// --- Pill drawing -----------------------------------------------------------

/// Pill body capped by four corner circles; corner size comes from the clamped
/// radius, identical to how helpers resolve it. `radius` of 0 gives a plain rect.
fn pill(x: f32, y: f32, w: f32, h: f32, radius: f32, color: Color) {
    let r = radius.min(w * 0.5).min(h * 0.5).max(0.0);
    if r <= 0.0 {
        draw_rectangle(x, y, w, h, color);
        return;
    }
    draw_rectangle(x + r, y, w - 2.0 * r, h, color);
    draw_rectangle(x, y + r, w, h - 2.0 * r, color);
    draw_circle(x + r, y + r, r, color);
    draw_circle(x + w - r, y + r, r, color);
    draw_circle(x + r, y + h - r, r, color);
    draw_circle(x + w - r, y + h - r, r, color);
}

/// Gradient pill fill whose right cap keeps a rounded end. The gradient runs
/// light at the top to dark at the bottom, then a thin translucent gloss band
/// polishes the top edge.
fn pill_fill(x: f32, y: f32, w: f32, h: f32, radius: f32, frac: f32, cols: (Color, Color)) {
    let (top, bottom) = cols;
    let fw = w * frac.clamp(0.0, 1.0);
    if fw <= 0.0 {
        return;
    }
    let r = radius.min(fw * 0.5).min(h * 0.5).max(0.0);
    // Split the fill body into 4 horizontal bands, top-light to bottom-dark.
    let bands: [Color; 4] = [
        mix(top, bottom, 0.0),
        mix(top, bottom, 0.33),
        mix(top, bottom, 0.66),
        mix(top, bottom, 0.87),
    ];
    let band_h = (h / 4.0).ceil();
    let cap_r = r.min(h * 0.5).min(fw * 0.5);
    let inner_w = if fw < 2.0 * cap_r {
        fw
    } else {
        fw - 2.0 * cap_r
    };
    for (i, color) in bands.iter().enumerate() {
        let by = y + i as f32 * (h / 4.0);
        let bh = band_h.min(h - i as f32 * (h / 4.0));
        if fw < 2.0 * cap_r {
            draw_rectangle(x, by, fw, bh, *color);
        } else {
            draw_rectangle(x + cap_r, by, inner_w, bh, *color);
        }
    }
    // Rounded end caps over the straight body.
    if fw >= 2.0 * cap_r {
        // Gradient end caps are approximated by the widest of the two cheeks.
        draw_circle(x + cap_r, y + cap_r, cap_r, mix(top, bottom, 0.5));
        draw_circle(x + fw - cap_r, y + cap_r, cap_r, mix(top, bottom, 0.5));
        draw_circle(x + cap_r, y + h - cap_r, cap_r, mix(top, bottom, 0.5));
        draw_circle(x + fw - cap_r, y + h - cap_r, cap_r, mix(top, bottom, 0.5));
    }
    // Gloss: thin translucent strip along the top edge.
    let gy = y + 0.5;
    let gh = (h * 0.35).max(1.0);
    let gw = fw.max(0.0);
    let gr = cap_r.min(gh * 0.5).min(gw * 0.5);
    pill(x, gy, gw, gh, gr, Color::new(1.0, 1.0, 1.0, 0.22));
}

fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

pub struct HudDrawSystem {
    store: Rc<EStore>,
    face: Texture2D,
    font: GameFont,
}

impl HudDrawSystem {
    pub fn new(_cfg: Rc<Config>, assets: &Assets, store: Rc<EStore>) -> Self {
        let face = assets.texture(images::Knight::Face);
        face.set_filter(FilterMode::Nearest);
        let base = assets.get_font(&TextStyle::new(FontTag::Body));
        let font = GameFont {
            size: 12,
            color: Color::new(0.96, 0.97, 1.0, 1.0),
            ..base
        };
        Self { store, face, font }
    }

    fn stats(&self) -> Option<HeroStats> {
        let knight = self.store.first::<Knight>()?;
        self.store.get_child::<HeroStats>(&knight).map(|r| *r)
    }

    fn dims(&self, text: &str) -> TextDimensions {
        measure_text(text, Some(&self.font.font), self.font.size, 1.0)
    }

    fn raw_text(&self, text: &str, x: f32, top: f32, color: Color) {
        let baseline = top + self.dims(text).offset_y;
        draw_text_ex(
            text,
            x,
            baseline,
            TextParams {
                font: Some(&self.font.font),
                font_size: self.font.size,
                font_scale: 1.0,
                color,
                ..Default::default()
            },
        );
    }

    /// Soft-shadow text: soft ink shadow, then the label in near-white.
    fn text(&self, text: &str, x: f32, top: f32) {
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
            self.raw_text(text, x + dx, top + dy, INK);
        }
        self.raw_text(text, x, top, self.font.color);
    }

    /// Rounded portrait plate: steel frame, dark back, face centered.
    fn portrait(&self) {
        let (x, y, s) = (MARGIN, MARGIN, PORTRAIT_SIZE);
        pill(x, y, s, s, PORTRAIT_RADIUS, FRAME);
        pill(
            x + 1.0,
            y + 1.0,
            s - 2.0,
            s - 2.0,
            PORTRAIT_RADIUS - 1.0,
            RIM,
        );
        pill(
            x + 2.0,
            y + 2.0,
            s - 4.0,
            s - 4.0,
            PORTRAIT_RADIUS - 2.0,
            TRACK,
        );
        let f = (s - FACE_SIZE) * 0.5;
        draw_texture_ex(
            &self.face,
            x + f,
            y + f,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(FACE_SIZE, FACE_SIZE)),
                ..Default::default()
            },
        );
    }

    /// Pill bar with steel frame, gradient fill and soft-shaded text.
    fn bar(&self, rect: Rect, frac: f32, style: FillStyle, label: &str, value: &str) {
        let Rect { x, y, w, h } = rect;
        pill(x, y, w, h, BAR_RADIUS, FRAME);
        pill(x + 1.0, y + 1.0, w - 2.0, h - 2.0, BAR_RADIUS - 1.0, RIM);
        pill(x + 2.0, y + 2.0, w - 4.0, h - 4.0, BAR_RADIUS - 2.0, TRACK);
        let (fx, fy, fw, fh) = (x + 2.0, y + 2.0, w - 4.0, h - 4.0);
        pill_fill(
            fx,
            fy,
            fw,
            fh,
            BAR_RADIUS - 2.0,
            frac,
            (style.light, style.dark),
        );
        let dims = self.dims(value);
        let top = y + (h - dims.height) * 0.5;
        self.text(label, x + 5.0, top);
        self.text(value, x + w - 5.0 - dims.width, top);
    }

    /// Thin XP progress line with the level in a medallion on its left end:
    /// gold ring, dark center, digit centered.
    fn xp_line(&self, frac: f32, level: u32) {
        // Medallion straddles the line's left end.
        let cx = BAR_X + MEDAL_R;
        let cy = XP_CENTER_Y;
        let bx = BAR_X + MEDAL_R * 2.0 - 2.0;
        let bw = BAR_W - (bx - BAR_X) - 1.0;
        let (y, h) = (XP_CENTER_Y - XP_H * 0.5, XP_H);
        let r = XP_H * 0.5;
        pill(bx, y, bw, h, r, FRAME);
        pill(bx + 1.0, y + 1.0, bw - 2.0, h - 2.0, r - 1.0, TRACK);
        let (fx, fy, fw, fh) = (bx + 1.0, y + 1.0, bw - 2.0, h - 2.0);
        pill_fill(fx, fy, fw, fh, r - 1.0, frac, (XP_LIGHT, XP_DARK));
        // Medallion: steel frame, gold ring, dark center.
        draw_circle(cx, cy, MEDAL_R, FRAME);
        draw_circle(cx, cy, MEDAL_R - 1.0, XP_DARK);
        draw_circle(cx, cy, MEDAL_R - 2.0, XP_LIGHT);
        draw_circle(cx, cy, MEDAL_R - 4.0, TRACK);
        let text = level.to_string();
        let dims = self.dims(&text);
        self.text(
            &text,
            cx - dims.width * 0.5,
            cy - self.dims(&text).offset_y * 0.5,
        );
    }

    pub fn draw(&self, _ctx: &Context) {
        let Some(stats) = self.stats() else {
            return;
        };

        self.portrait();

        let hp = format!("{}/{}", stats.hp, stats.max_hp);
        let hp_frac = stats.hp as f32 / stats.max_hp.max(1) as f32;
        self.bar(
            Rect::new(BAR_X, HP_Y, BAR_W, BAR_H),
            hp_frac,
            FillStyle {
                light: HP_LIGHT,
                dark: HP_DARK,
            },
            "HP",
            &hp,
        );

        let mp = format!("{}/{}", stats.mp, stats.max_mp);
        let mp_frac = stats.mp as f32 / stats.max_mp.max(1) as f32;
        self.bar(
            Rect::new(BAR_X, MP_Y, BAR_W, BAR_H),
            mp_frac,
            FillStyle {
                light: MP_LIGHT,
                dark: MP_DARK,
            },
            "MP",
            &mp,
        );

        let xp_frac = stats.xp as f32 / stats.max_xp.max(1) as f32;
        self.xp_line(xp_frac, stats.level);
    }
}
