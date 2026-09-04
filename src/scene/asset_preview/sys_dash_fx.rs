use crate::entity::knight::{Dash, Knight, DASH_DURATION};
use crate::systems::Draw;
use crate::{shader, Animation, Assets, Context, EStore};
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use macroquad::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

const GHOST_COUNT: usize = 3;
const GHOST_SPACING: f32 = 13.0;
const GHOST_BASE_ALPHA: f32 = 0.7;

struct Ghost {
    texture: Texture2D,
    position: Vec2,
    source: Rect,
    dest_size: Vec2,
    flip_x: bool,
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

fn load_fx_material(vertex: &str, fragment: &str, params: MaterialParams) -> Option<Material> {
    match load_material(ShaderSource::Glsl { vertex, fragment }, params) {
        Ok(material) => Some(material),
        Err(err) => {
            warn!("dash fx shader failed to compile: {}", err);
            None
        }
    }
}

pub struct DashFxDrawSystem {
    store: Rc<EStore>,
    afterimage: Option<Material>,
    dash_color: Rc<Cell<Color>>,
}

impl DashFxDrawSystem {
    pub fn new(store: Rc<EStore>, assets: &Assets, dash_color: Rc<Cell<Color>>) -> Self {
        let vertex = assets.shader(shader::Shader::DashFxVert);

        let alpha = BlendState::new(
            Equation::Add,
            BlendFactor::Value(BlendValue::SourceAlpha),
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
        );
        let keep_alpha = BlendState::new(Equation::Add, BlendFactor::Zero, BlendFactor::One);

        let afterimage = load_fx_material(
            &vertex,
            &assets.shader(shader::Shader::DashAfterimageFrag),
            MaterialParams {
                pipeline_params: PipelineParams {
                    color_blend: Some(alpha),
                    alpha_blend: Some(keep_alpha),
                    ..Default::default()
                },
                uniforms: vec![UniformDesc::new("tint", UniformType::Float4)],
                ..Default::default()
            },
        );

        Self {
            store,
            afterimage,
            dash_color,
        }
    }

    fn draw_afterimage(
        &self,
        ghost: &Ghost,
        dir: Vec2,
        progress: f32,
        color: Color,
    ) {
        let Some(material) = &self.afterimage else { return };

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
}

impl Draw for DashFxDrawSystem {
    fn draw(&self, _ctx: &Context) {
        let Some(knight) = self.store.first::<Knight>() else {
            return;
        };
        let Some(dash) = self.store.get_child::<Dash>(&knight) else {
            return;
        };
        let Some(animation) = self.store.get_child::<Animation>(&knight) else {
            return;
        };

        let dir = dash.dir;
        let progress = (1.0 - dash.time / DASH_DURATION).clamp(0.0, 1.0);
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
