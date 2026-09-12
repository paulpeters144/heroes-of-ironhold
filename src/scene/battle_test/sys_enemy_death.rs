//! Hellfire immolation when an enemy dies: the body burns from the feet up,
//! embers spiral out of the flames, and the demon's soul tears free and rises.
use crate::entity::enemy::RamHead;
use crate::entity::impact_frame::ImpactFrame;
use crate::events::EnemyDeathEvent;
use crate::systems::System;
use crate::{shader, Animation, Assets, Context, EStore, EventBus, StaticImage, SubCollection};
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use macroquad::prelude::*;
use macroquad::rand::gen_range;
use std::cell::RefCell;
use std::rc::Rc;

/// Total effect length; the body itself is gone after `BURN_TIME`.
const DURATION: f32 = 1.5;
const BURN_TIME: f32 = 0.95;
const EMBER_COUNT: usize = 26;
const ASH_COUNT: usize = 12;
const SOUL_DELAY: f32 = 0.3;
const TRAIL_MAX: usize = 16;

const EMBER_HOT: Color = Color::new(1.0, 0.85, 0.4, 1.0);
const EMBER_MID: Color = Color::new(1.0, 0.45, 0.1, 1.0);
const EMBER_LOW: Color = Color::new(0.75, 0.12, 0.02, 1.0);
const ASH: Color = Color::new(0.16, 0.12, 0.1, 1.0);
const SOUL_CORE: Color = Color::new(0.9, 1.0, 0.97, 1.0);
const SOUL_GLOW: Color = Color::new(0.35, 0.95, 0.65, 1.0);
const SCORCH_DARK: Color = Color::new(0.4, 0.07, 0.02, 1.0);
const SCORCH_HOT: Color = Color::new(0.9, 0.25, 0.05, 1.0);

/// Rises out of the flames, flickering between hot and cooling colors.
struct Ember {
    pos: Vec2,
    vel: Vec2,
    size: f32,
    /// Ignites when the burn front reaches it, so embers appear bottom-up.
    delay: f32,
    phase: f32,
    hot: bool,
}

/// Charred flake drifting back down.
struct Ash {
    pos: Vec2,
    vel: Vec2,
    size: f32,
    delay: f32,
    phase: f32,
}

/// The demon's escaping soul: a pulsing orb with a fading trail.
struct Soul {
    pos: Vec2,
    vel: Vec2,
    trail: Vec<Vec2>,
}

struct DeathFx {
    texture: Texture2D,
    source: Rect,
    center: Vec2,
    position: Vec2,
    dest_size: Vec2,
    flip_x: bool,
    age: f32,
    embers: Vec<Ember>,
    ash: Vec<Ash>,
    soul: Soul,
}

pub struct EnemyDeathSystem {
    store: Rc<EStore>,
    queue: Rc<RefCell<Vec<EnemyDeathEvent>>>,
    active: Vec<DeathFx>,
    material: Option<Material>,
    _subs: Rc<SubCollection>,
}

fn mix_color(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

impl EnemyDeathSystem {
    pub fn new(store: Rc<EStore>, bus: Rc<EventBus>, assets: &Assets) -> Self {
        let queue = Rc::new(RefCell::new(Vec::new()));
        let subs = Rc::new(SubCollection::new());
        {
            let queue = queue.clone();
            subs.on::<EnemyDeathEvent>(&bus, move |event: &EnemyDeathEvent| {
                queue.borrow_mut().push(event.clone());
            });
        }
        let material = load_immolation_material(assets);
        Self {
            store,
            queue,
            active: Vec::new(),
            material,
            _subs: subs,
        }
    }

    /// Snapshot the enemy's visual state, then remove the entity. Read out
    /// `get_by_id` inside a scope before `remove` so the read guard drops
    /// first (parking_lot locks are not re-entrant).
    fn begin_fx(&mut self, enemy: u64) {
        let (ram_ref, mut fx) = {
            let Some(ram) = self.store.get_by_id::<RamHead>(enemy) else {
                return;
            };
            let ram_ref = ram.entity_ref();

            // The demon dies mid-hit: freeze it in the hit pose (the impact
            // image), not the idle/walk body frame.
            let hit = self
                .store
                .get_child::<ImpactFrame>(&ram)
                .and_then(|frame| self.store.get_child::<StaticImage>(&frame))
                .filter(|image| image.visible);

            let fx = match hit {
                Some(image) => {
                    let dest_size = image.size * image.scale;
                    DeathFx {
                        texture: image.source.clone(),
                        source: Rect::new(0.0, 0.0, image.source.width(), image.source.height()),
                        center: image.position + dest_size * 0.5,
                        position: image.position,
                        dest_size,
                        flip_x: image.flip_x,
                        age: 0.0,
                        embers: Vec::new(),
                        ash: Vec::new(),
                        soul: Soul {
                            pos: image.position + dest_size * 0.5,
                            vel: Vec2::ZERO,
                            trail: Vec::new(),
                        },
                    }
                }
                None => {
                    let Some(body) = self.store.get_child::<Animation>(&ram) else {
                        return;
                    };
                    let source = Rect::new(
                        body.current_frame as f32 * body.frame_width,
                        0.0,
                        body.frame_width,
                        body.frame_height,
                    );
                    let dest_size = if body.dest_size == Vec2::ZERO {
                        vec2(body.frame_width, body.frame_height)
                    } else {
                        body.dest_size
                    } * body.scale;
                    DeathFx {
                        texture: body.source.clone(),
                        source,
                        center: body.position + dest_size * 0.5,
                        position: body.position,
                        dest_size,
                        flip_x: body.flip_x,
                        age: 0.0,
                        embers: Vec::new(),
                        ash: Vec::new(),
                        soul: Soul {
                            pos: body.position + dest_size * 0.5,
                            vel: Vec2::ZERO,
                            trail: Vec::new(),
                        },
                    }
                }
            };
            (ram_ref, fx)
        };
        self.store.remove(&[ram_ref]);
        self.spawn_particles(&mut fx);
        self.active.push(fx);
    }

    /// Embers ignite when the wavy burn front reaches their height, so the
    /// shower of sparks sweeps upward together with the flames.
    fn spawn_particles(&self, fx: &mut DeathFx) {
        for _ in 0..EMBER_COUNT {
            let r0 = gen_range(0.0, 1.0);
            let r1 = gen_range(0.0, 1.0);
            let r2 = gen_range(0.0, 1.0);
            let pos = fx.position + vec2(r0 * fx.dest_size.x, r1 * fx.dest_size.y);
            fx.embers.push(Ember {
                pos,
                vel: vec2((r0 - 0.5) * 16.0, -(16.0 + r2 * 26.0)),
                size: 0.7 + r2 * 1.3,
                delay: (1.0 - r1) * BURN_TIME * 0.75 + r2 * 0.12,
                phase: r0 * std::f32::consts::TAU,
                hot: r2 > 0.6,
            });
        }
        for _ in 0..ASH_COUNT {
            let r0 = gen_range(0.0, 1.0);
            let r1 = gen_range(0.0, 1.0);
            let r2 = gen_range(0.0, 1.0);
            fx.ash.push(Ash {
                pos: fx.position + vec2(r0 * fx.dest_size.x, r1 * fx.dest_size.y),
                vel: vec2((r0 - 0.5) * 10.0, 6.0 + r2 * 12.0),
                size: 0.8 + r2 * 1.2,
                delay: (1.0 - r1) * BURN_TIME * 0.5 + 0.15 + r2 * 0.2,
                phase: r1 * std::f32::consts::TAU,
            });
        }
    }
}

// Same sprite-processing material pipeline as the knight afterimage.
fn load_immolation_material(assets: &Assets) -> Option<Material> {
    let vertex = assets.shader(shader::Shader::DashFxVert);
    let fragment = assets.shader(shader::Shader::DemonDeathFrag);

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
                UniformDesc::new("rect", UniformType::Float4),
                UniformDesc::new("progress", UniformType::Float1),
                UniformDesc::new("time", UniformType::Float1),
            ],
            ..Default::default()
        },
    ) {
        Ok(material) => Some(material),
        Err(err) => {
            warn!("demon death shader failed to compile: {}", err);
            None
        }
    }
}

impl System for EnemyDeathSystem {
    fn update(&mut self, ctx: &mut Context) {
        let dt = ctx.dt;
        let events: Vec<EnemyDeathEvent> = self.queue.borrow_mut().drain(..).collect();
        for event in events {
            self.begin_fx(event.enemy);
        }
        for fx in self.active.iter_mut() {
            fx.age += dt;
            for ember in fx.embers.iter_mut() {
                if fx.age < ember.delay {
                    continue;
                }
                // Sparks ride the heat: accelerating upward, swaying, cooling.
                ember.vel.y -= 26.0 * dt;
                ember.pos.x += (fx.age * 7.0 + ember.phase).sin() * 22.0 * dt;
                ember.pos += ember.vel * dt;
                ember.size = (ember.size - dt * 0.5).max(0.0);
            }
            for flake in fx.ash.iter_mut() {
                if fx.age < flake.delay {
                    continue;
                }
                flake.pos.x += (fx.age * 3.0 + flake.phase).sin() * 10.0 * dt;
                flake.pos += flake.vel * dt;
            }
            // The soul hangs on for a beat, then tears free and climbs.
            if fx.age > SOUL_DELAY {
                let st = fx.age - SOUL_DELAY;
                fx.soul.vel.y = (-14.0 - st * 110.0).max(-95.0);
                fx.soul.vel.x = (st * 5.0).sin() * 10.0;
                fx.soul.pos += fx.soul.vel * dt;
                fx.soul.trail.push(fx.soul.pos);
                if fx.soul.trail.len() > TRAIL_MAX {
                    fx.soul.trail.remove(0);
                }
            }
        }
        self.active.retain(|fx| fx.age < DURATION);
    }

    fn draw(&self, _ctx: &Context) {
        for fx in self.active.iter() {
            let t = (fx.age / DURATION).clamp(0.0, 1.0);

            // Ground scorch: a smouldering stain that flares then fades.
            let grow = (fx.age / 0.3).min(1.0);
            let scorch_fade = 1.0 - t * t;
            let rx = fx.dest_size.x * 0.42 * (0.6 + 0.4 * grow);
            let ground_y = fx.position.y + fx.dest_size.y;
            draw_ellipse(
                fx.center.x,
                ground_y,
                rx,
                rx * 0.28,
                0.0,
                Color::new(SCORCH_DARK.r, SCORCH_DARK.g, SCORCH_DARK.b, 0.5 * grow * scorch_fade),
            );
            draw_ellipse(
                fx.center.x,
                ground_y,
                rx * 0.6,
                rx * 0.17,
                0.0,
                Color::new(SCORCH_HOT.r, SCORCH_HOT.g, SCORCH_HOT.b, 0.35 * grow * scorch_fade),
            );

            // The burning body.
            if let Some(material) = self.material.as_ref() {
                let tw = fx.texture.width();
                let th = fx.texture.height();
                gl_use_material(material);
                material.set_uniform(
                    "rect",
                    vec4(
                        fx.source.x / tw,
                        fx.source.y / th,
                        fx.source.w / tw,
                        fx.source.h / th,
                    ),
                );
                material.set_uniform("progress", (fx.age / BURN_TIME).min(1.0));
                material.set_uniform("time", fx.age);
                draw_texture_ex(
                    &fx.texture,
                    fx.position.x,
                    fx.position.y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(fx.dest_size),
                        source: Some(fx.source),
                        flip_x: fx.flip_x,
                        ..Default::default()
                    },
                );
                gl_use_default_material();
            }

            // Ignition flash: a fast-expanding ring of fire.
            let flash = (fx.age / 0.22).min(1.0);
            if flash < 1.0 {
                let ease = 1.0 - (1.0 - flash) * (1.0 - flash);
                let r = 6.0 + 34.0 * ease;
                let a = (1.0 - flash) * 0.8;
                draw_circle_lines(fx.center.x, fx.center.y, r, 2.0, Color::new(1.0, 0.6, 0.2, a));
                draw_circle(
                    fx.center.x,
                    fx.center.y,
                    r * 0.55,
                    Color::new(1.0, 0.85, 0.5, a * 0.35),
                );
            }

            // Embers: glow halo, then a hot core that cools as it climbs.
            let global_fade = (1.0 - t).powf(0.8);
            for ember in fx.embers.iter() {
                if fx.age < ember.delay {
                    continue;
                }
                let cool = ((fx.age - ember.delay) / 0.8).clamp(0.0, 1.0);
                let col = if ember.hot {
                    mix_color(EMBER_HOT, EMBER_MID, cool)
                } else {
                    mix_color(EMBER_MID, EMBER_LOW, cool)
                };
                let a = global_fade * (1.0 - cool * 0.6);
                draw_circle(
                    ember.pos.x,
                    ember.pos.y,
                    ember.size * 2.2,
                    Color::new(col.r, col.g, col.b, a * 0.18),
                );
                draw_circle(
                    ember.pos.x,
                    ember.pos.y,
                    ember.size,
                    Color::new(col.r, col.g, col.b, a),
                );
            }

            // Ash flakes settle back down through the smoke.
            for flake in fx.ash.iter() {
                if fx.age < flake.delay {
                    continue;
                }
                let fade = (1.0 - (fx.age - flake.delay) / 0.9).clamp(0.0, 1.0);
                draw_circle(
                    flake.pos.x,
                    flake.pos.y,
                    flake.size,
                    Color::new(ASH.r, ASH.g, ASH.b, 0.6 * fade),
                );
            }

            // The escaped soul: trail, halo, and a pulsing core.
            let st = fx.age - SOUL_DELAY;
            if st > 0.0 {
                let fade = (1.0 - (st / (DURATION - SOUL_DELAY)).powf(1.5)).max(0.0);
                let pulse = 1.0 + 0.12 * (st * 18.0).sin();
                let trail_len = fx.soul.trail.len();
                for (i, p) in fx.soul.trail.iter().enumerate() {
                    let f = (i + 1) as f32 / trail_len as f32;
                    draw_circle(
                        p.x,
                        p.y,
                        (1.0 + 2.2 * f) * pulse,
                        Color::new(SOUL_GLOW.r, SOUL_GLOW.g, SOUL_GLOW.b, 0.12 * f * fade),
                    );
                }
                draw_circle(
                    fx.soul.pos.x,
                    fx.soul.pos.y,
                    7.0 * pulse,
                    Color::new(SOUL_GLOW.r, SOUL_GLOW.g, SOUL_GLOW.b, 0.16 * fade),
                );
                draw_circle(
                    fx.soul.pos.x,
                    fx.soul.pos.y,
                    3.6 * pulse,
                    Color::new(SOUL_GLOW.r, SOUL_GLOW.g, SOUL_GLOW.b, 0.45 * fade),
                );
                draw_circle(
                    fx.soul.pos.x,
                    fx.soul.pos.y,
                    1.8 * pulse,
                    Color::new(SOUL_CORE.r, SOUL_CORE.g, SOUL_CORE.b, 0.95 * fade),
                );
            }
        }
    }
}
