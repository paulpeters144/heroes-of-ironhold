use super::Drawable;
use crate::entity::{Animation, Knight, PlayerOne, Shield, StaticImage, Sword};
use crate::{shader, Assets, EStore};
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use macroquad::prelude::*;
use std::rc::Rc;

const BRIGHT_GOLD: Color = Color::new(1.0, 0.92, 0.6, 1.0);

#[derive(Clone, Debug)]
pub struct ConsecrationAura {
    pub knight_id: u64,
    pub z_idx: f32,
    pub visible: bool,
}

pub fn load_aura_material(assets: &Assets) -> Option<Material> {
    let vertex = assets.shader(shader::Shader::DashFxVert);
    let fragment = assets.shader(shader::Shader::KnightGoldenGlowFrag);

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
            uniforms: vec![
                UniformDesc::new("tint", UniformType::Float4),
                UniformDesc::new("trail_dir", UniformType::Float1),
                UniformDesc::new("flying", UniformType::Float1),
                UniformDesc::new("appear", UniformType::Float1),
                UniformDesc::new("fade", UniformType::Float1),
                UniformDesc::new("alpha", UniformType::Float1),
            ],
            ..Default::default()
        },
    ) {
        Ok(material) => Some(material),
        Err(err) => {
            warn!("consecration aura shader failed to compile: {}", err);
            None
        }
    }
}

impl ConsecrationAura {
    pub fn draw_with_material(&self, material: &Material, store: &Rc<EStore>) {
        if !self.visible {
            return;
        }

        let Some(knight_ref) = store
            .first::<PlayerOne>()
            .and_then(|p| store.get_child::<Knight>(&p))
            .map(|k| k.entity_ref())
        else {
            return;
        };

        if knight_ref.id() != self.knight_id {
            return;
        }

        let pulse = 0.7 + 0.3 * (get_time() * 3.0).sin() as f32;

        gl_use_material(material);
        material.set_uniform(
            "tint",
            vec4(BRIGHT_GOLD.r, BRIGHT_GOLD.g, BRIGHT_GOLD.b, 1.0),
        );
        material.set_uniform("trail_dir", 0.0_f32);
        material.set_uniform("flying", 0.0_f32);
        material.set_uniform("appear", 1.0_f32);
        material.set_uniform("fade", 0.0_f32);
        material.set_uniform("alpha", pulse * 0.6_f32);

        if let Some(anim) = store
            .get_by_id::<Knight>(knight_ref.id())
            .and_then(|k| store.get_child::<Animation>(&k))
        {
            let source = Rect::new(
                anim.current_frame as f32 * anim.frame_width,
                0.0,
                anim.frame_width,
                anim.frame_height,
            );
            draw_texture_ex(
                &anim.source,
                anim.position.x,
                anim.position.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(anim.dest_size * anim.scale),
                    source: Some(source),
                    flip_x: anim.flip_x,
                    ..Default::default()
                },
            );
        }

        if let Some(image) = store
            .get_by_id::<Knight>(knight_ref.id())
            .and_then(|k| store.get_child::<Shield>(&k))
            .and_then(|s| store.get_child::<StaticImage>(&s))
        {
            if image.visible {
                draw_texture_ex(
                    &image.source,
                    image.position.x,
                    image.position.y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(image.size * image.scale),
                        flip_x: image.flip_x,
                        ..Default::default()
                    },
                );
            }
        }

        if let Some(anim) = store
            .get_by_id::<Knight>(knight_ref.id())
            .and_then(|k| store.get_child::<Sword>(&k))
            .and_then(|s| store.get_child::<Animation>(&s))
        {
            if anim.visible {
                let source = Rect::new(
                    anim.current_frame as f32 * anim.frame_width,
                    0.0,
                    anim.frame_width,
                    anim.frame_height,
                );
                draw_texture_ex(
                    &anim.source,
                    anim.position.x,
                    anim.position.y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(anim.dest_size * anim.scale),
                        source: Some(source),
                        flip_x: anim.flip_x,
                        ..Default::default()
                    },
                );
            }
        }

        gl_use_default_material();
    }
}

impl Drawable for ConsecrationAura {
    fn draw(&self) {
        // Aura drawing requires the material and store, which are held by the system.
        // The system calls draw_with_material directly instead of using this trait method.
    }

    fn zdx(&self) -> f32 {
        self.z_idx
    }
}
