use crate::entity::dash::Dash;
use crate::entity::knight::{
    Knight, DOUBLE_TAP_WINDOW, IDLE_FRAME, SWIPE_FRAME, THRUST_FRAME,
};
use crate::input::{self, Input};
use crate::systems::{Draw, Update};
use crate::ui::draw_rounded_rect;
use crate::util::view_scale;
use crate::{shader, Animation, Assets, Config, Context, EStore, FontTag, GameFont, TextStyle};
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use macroquad::prelude::*;
use pico_entity_store::store::IntoChild;
use std::cell::Cell;
use std::rc::Rc;

const GHOST_COUNT: usize = 3;
const GHOST_SPACING: f32 = 13.0;
const GHOST_BASE_ALPHA: f32 = 0.7;

// Dash UI layout (virtual 640x360): a square well on the left edge, below the
// top-left HUD block (portrait + HP/MP bars + XP line end around y ~= 58).
const DASH_X: f32 = 8.0;
const DASH_Y: f32 = 66.0;
const DASH_SIZE: f32 = 26.0;
const DASH_RADIUS: f32 = 3.0;
const CHARGE_INSET_X: f32 = 2.0;
const CHARGE_INSET_Y: f32 = 1.0;

// Palette: match the HUD steel frame / rim / track look.
const FRAME: Color = Color::new(0.08, 0.09, 0.12, 1.0);
const RIM: Color = Color::new(0.45, 0.48, 0.56, 1.0);
const TRACK: Color = Color::new(0.2, 0.192, 0.243, 1.0);
const CHARGE_TEXT: Color = Color::new(0.86, 0.88, 0.96, 1.0);
const ICON_COLOR: Color = Color::new(0.82, 0.84, 0.92, 1.0);
const SWEEP: Color = Color::new(0.02, 0.02, 0.05, 0.55);

struct Ghost {
    texture: Texture2D,
    position: Vec2,
    source: Rect,
    dest_size: Vec2,
    flip_x: bool,
}

#[derive(Clone)]
pub struct DashSystem {
    store: Rc<EStore>,
    afterimage: Rc<Option<Material>>,
    dash_color: Rc<Cell<Color>>,
    font: GameFont,
    cfg: Rc<Config>,
    last_tap: Option<Input>,
    tap_timer: f32,
}

pub fn outfit_dominant_color(texture: &Texture2D) -> Color {
    let image = texture.get_texture_data();
    let data = image.get_image_data();
    let width = image.width();
    let height = image.height();

    let x0 = width / 4;
    let x1 = width * 3 / 4;
    let y0 = height / 4;
    let y1 = height * 3 / 4;

    let mut hist: std::collections::HashMap<(u8, u8, u8), u32> = std::collections::HashMap::new();
    for y in y0..y1 {
        for x in x0..x1 {
            let px = &data[y * width + x];
            if px[3] < 16 {
                continue;
            }
            // Skip near-black outline pixels so they don't dominate the sample.
            if px[0].max(px[1]).max(px[2]) < 24 {
                continue;
            }
            let key = (px[0] >> 3, px[1] >> 3, px[2] >> 3);
            *hist.entry(key).or_insert(0) += 1;
        }
    }

    let best = hist
        .iter()
        .max_by_key(|(_, &count)| count)
        .map(|(&k, _)| k)
        .unwrap_or((255 >> 3, 255 >> 3, 255 >> 3));

    let (r, g, b) = (best.0 << 3, best.1 << 3, best.2 << 3);
    let max = (r.max(g)).max(b).max(1) as f32;

    Color::new(r as f32 / max, g as f32 / max, b as f32 / max, 1.0)
}

fn load_afterimage_material(assets: &Assets) -> Option<Material> {
    let vertex = assets.shader(shader::Shader::DashFxVert);
    let fragment = assets.shader(shader::Shader::DashAfterimageFrag);

    let alpha = BlendState::new(
        Equation::Add,
        BlendFactor::Value(BlendValue::SourceAlpha),
        BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
    );
    let keep_alpha = BlendState::new(Equation::Add, BlendFactor::Zero, BlendFactor::One);

    match load_material(
        ShaderSource::Glsl {
            vertex: &vertex,
            fragment: &fragment,
        },
        MaterialParams {
            pipeline_params: PipelineParams {
                color_blend: Some(alpha),
                alpha_blend: Some(keep_alpha),
                ..Default::default()
            },
            uniforms: vec![UniformDesc::new("tint", UniformType::Float4)],
            ..Default::default()
        },
    ) {
        Ok(material) => Some(material),
        Err(err) => {
            warn!("dash fx shader failed to compile: {}", err);
            None
        }
    }
}

impl DashSystem {
    pub fn new(store: Rc<EStore>, assets: &Assets, dash_color: Rc<Cell<Color>>, cfg: Rc<Config>) -> Self {
        let base = assets.get_font(&TextStyle::new(FontTag::Body));
        let font = GameFont {
            size: 12,
            color: CHARGE_TEXT,
            ..base
        };
        Self {
            store,
            afterimage: Rc::new(load_afterimage_material(assets)),
            dash_color,
            font,
            cfg,
            last_tap: None,
            tap_timer: 0.0,
        }
    }

    fn knight_id(&self) -> Option<u64> {
        self.store.first::<Knight>().map(|k| k.id())
    }

    fn dash(&self) -> Option<Dash> {
        let knight = self.store.get_by_id::<Knight>(self.knight_id()?)?;
        self.store.get_child::<Dash>(&knight).map(|d| *d)
    }

    fn draw_afterimage(&self, ghost: &Ghost, dir: Vec2, progress: f32, color: Color) {
        let Some(material) = self.afterimage.as_ref() else {
            return;
        };

        gl_use_material(material);
        for i in 0..GHOST_COUNT {
            let t = i as f32 / GHOST_COUNT as f32;
            let alpha = GHOST_BASE_ALPHA * (1.0 - t) * (1.0 - t) * progress;
            let offset = dir * (GHOST_SPACING * (i as f32 + 1.0));

            material.set_uniform("tint", vec4(color.r, color.g, color.b, alpha));
            draw_texture_ex(
                &ghost.texture,
                ghost.position.x - offset.x,
                ghost.position.y - offset.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(ghost.dest_size),
                    source: Some(ghost.source),
                    flip_x: ghost.flip_x,
                    ..Default::default()
                },
            );
        }
        gl_use_default_material();
    }

    pub fn draw_ui(&self, _ctx: &Context) {
        let Some(dash) = self.dash() else {
            return;
        };

        let x = DASH_X;
        let y = DASH_Y;

        draw_rounded_rect(x, y, DASH_SIZE, DASH_SIZE, DASH_RADIUS, FRAME);
        draw_rounded_rect(
            x + 1.0,
            y + 1.0,
            DASH_SIZE - 2.0,
            DASH_SIZE - 2.0,
            DASH_RADIUS - 1.0,
            RIM,
        );
        draw_rounded_rect(
            x + 2.0,
            y + 2.0,
            DASH_SIZE - 4.0,
            DASH_SIZE - 4.0,
            DASH_RADIUS - 2.0,
            TRACK,
        );

        let cx = x + DASH_SIZE * 0.5;
        let cy = y + DASH_SIZE * 0.5;

        Self::dash_icon(cx, cy, DASH_SIZE - 12.0);

        let text = dash.charges.to_string();
        let scale = view_scale::view_scale(self.cfg.v_width, self.cfg.v_height).0;
        let (font_size, font_scale) = view_scale::crisp_text_params(self.font.size, scale);
        let dims = measure_text(&text, Some(&self.font.font), font_size, font_scale);
        let tx = view_scale::snap_to_pixel(
            x + DASH_SIZE - CHARGE_INSET_X - dims.width,
            scale,
        );
        let ty = view_scale::snap_to_pixel(y + CHARGE_INSET_Y + dims.offset_y, scale);
        draw_text_ex(
            &text,
            tx,
            ty,
            TextParams {
                font: Some(&self.font.font),
                font_size,
                font_scale,
                color: CHARGE_TEXT,
                ..Default::default()
            },
        );

        if dash.charges < dash.cfg.max_charges && dash.recovery > 0.0 {
            let frac = (dash.recovery / dash.cfg.recover_secs).clamp(0.0, 1.0);
            let r = (DASH_SIZE - 4.0) * 0.5;
            Self::radial_sweep(cx, cy, r, frac, SWEEP);
        }
    }

    /// Double chevron (">>"), a dodge/dash glyph.
    fn dash_icon(cx: f32, cy: f32, s: f32) {
        let r = s * 0.34;
        let gap = s * 0.16;
        for off in [-gap, gap] {
            let bx = cx + off + r * 0.5;
            draw_line(bx - r, cy - r, bx, cy, 2.0, ICON_COLOR);
            draw_line(bx, cy, bx - r, cy + r, 2.0, ICON_COLOR);
        }
    }

    /// Darkened wedge anchored at 12 o'clock, sweeping clockwise. `frac` is
    /// the remaining cooldown fraction: 1.0 paints a full disc, 0.0 nothing.
    fn radial_sweep(cx: f32, cy: f32, r: f32, frac: f32, color: Color) {
        if frac <= 0.0 {
            return;
        }
        let full = frac * std::f32::consts::TAU;
        let steps = ((32.0 * frac).ceil() as usize).max(1);
        let start = -std::f32::consts::FRAC_PI_2;
        for i in 0..steps {
            let a0 = start + full * (i as f32 / steps as f32);
            let a1 = start + full * ((i + 1) as f32 / steps as f32);
            draw_triangle(
                vec2(cx, cy),
                vec2(cx + a0.cos() * r, cy + a0.sin() * r),
                vec2(cx + a1.cos() * r, cy + a1.sin() * r),
                color,
            );
        }
    }
}

impl Update for DashSystem {
    fn update(&mut self, ctx: &mut Context) {
        self.tap_timer = (self.tap_timer - ctx.dt).max(0.0);

        let Some(knight_id) = self.knight_id() else {
            return;
        };

        let has_dash = self
            .store
            .get_by_id::<Knight>(knight_id)
            .and_then(|k| self.store.get_child::<Dash>(&k))
            .is_some();
        if !has_dash {
            if let Some(knight) = self.store.get_by_id::<Knight>(knight_id) {
                self.store.add(knight, &[Dash::knight().into_child()]);
            }
        }

        let mut dash = self.dash().unwrap_or_else(Dash::knight);

        let attacking = self
            .store
            .get_by_id::<Knight>(knight_id)
            .and_then(|k| self.store.get_child::<Animation>(&k))
            .map(|a| matches!(a.current_frame, THRUST_FRAME | SWIPE_FRAME))
            .unwrap_or(false);

        if !attacking && dash.time <= 0.0 && dash.charges > 0 {
            let taps = [
                (Input::Left, Vec2::new(-1.0, 0.0)),
                (Input::Right, Vec2::new(1.0, 0.0)),
                (Input::Up, Vec2::new(0.0, -1.0)),
                (Input::Down, Vec2::new(0.0, 1.0)),
            ];
            for (input, dir) in taps {
                if input::down_once(input) {
                    if self.last_tap == Some(input) && self.tap_timer > 0.0 {
                        dash.dir = dir;
                        dash.time = dash.cfg.duration;
                        dash.charges -= 1;
                        if dash.recovery <= 0.0 {
                            dash.recovery = dash.cfg.recover_secs;
                        }
                        self.last_tap = None;
                        self.tap_timer = 0.0;
                    } else {
                        self.last_tap = Some(input);
                        self.tap_timer = DOUBLE_TAP_WINDOW;
                    }
                    break;
                }
            }
        }

        if dash.time > 0.0 {
            dash.time -= ctx.dt;
            let dir = dash.dir;
            if let Some(anim_ref) = self
                .store
                .get_by_id::<Knight>(knight_id)
                .and_then(|k| self.store.get_child::<Animation>(&k))
                .map(|a| a.entity_ref())
            {
                self.store.update::<Animation, _>(&anim_ref, |a| {
                    a.position.x += dir.x * dash.cfg.speed * ctx.dt;
                    a.position.y += dir.y * dash.cfg.speed * ctx.dt;
                    a.current_frame = IDLE_FRAME;
                });
            }
        }

        if dash.charges < dash.cfg.max_charges {
            dash.recovery -= ctx.dt;
            if dash.recovery <= 0.0 {
                dash.charges += 1;
                if dash.charges < dash.cfg.max_charges {
                    dash.recovery = dash.cfg.recover_secs;
                } else {
                    dash.recovery = 0.0;
                }
            }
        } else {
            dash.recovery = 0.0;
        }

        if let Some(dash_ref) = self
            .store
            .get_by_id::<Knight>(knight_id)
            .and_then(|k| self.store.get_child::<Dash>(&k))
            .map(|d| d.entity_ref())
        {
            self.store.update::<Dash, _>(&dash_ref, |d| *d = dash);
        }
    }
}

impl Draw for DashSystem {
    fn draw(&self, _ctx: &Context) {
        let Some(knight_id) = self.knight_id() else {
            return;
        };
        let Some(dash) = self.dash() else {
            return;
        };
        if dash.time <= 0.0 {
            return;
        }

        let Some(knight) = self.store.get_by_id::<Knight>(knight_id) else {
            return;
        };
        let Some(animation) = self.store.get_child::<Animation>(&knight) else {
            return;
        };

        let dir = dash.dir;
        let progress = (1.0 - dash.time / dash.cfg.duration).clamp(0.0, 1.0);
        let color = self.dash_color.get();
        let source = Rect::new(
            animation.current_frame as f32 * animation.frame_width,
            0.0,
            animation.frame_width,
            animation.frame_height,
        );
        let dest_size = if animation.dest_size == Vec2::ZERO {
            vec2(animation.frame_width, animation.frame_height)
        } else {
            animation.dest_size
        } * animation.scale;

        self.draw_afterimage(
            &Ghost {
                texture: animation.source.clone(),
                position: animation.position,
                source,
                dest_size,
                flip_x: animation.flip_x,
            },
            dir,
            progress,
            color,
        );
    }
}
