//! Evaporation disintegration when an enemy dies.
use crate::entity::enemy::RamHead;
use crate::entity::impact_frame::ImpactFrame;
use crate::events::EnemyDeathEvent;
use crate::systems::{Draw, Update};
use crate::{shader, Animation, Assets, Context, EStore, EventBus, StaticImage, SubCollection};
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use macroquad::prelude::*;
use macroquad::rand::gen_range;
use std::cell::RefCell;
use std::rc::Rc;

const DURATION: f32 = 1.0;
/// Downward pull: the body is dragged back into the rift, not flung upward.
const PULL: f32 = 5.0;
const EDGE: f32 = 0.18;
const EMBER_COUNT: usize = 24;
const WISP_COUNT: usize = 8;

const VOID_DEEP: Color = Color::new(0.14, 0.0, 0.26, 1.0);
const VOID_PURPLE: Color = Color::new(0.36, 0.05, 0.6, 1.0);

struct Spark {
    pos: Vec2,
    vel: Vec2,
    size: f32,
}

struct DeathFx {
    texture: Texture2D,
    source: Rect,
    center: Vec2,
    position: Vec2,
    dest_size: Vec2,
    flip_x: bool,
    age: f32,
    sparks: Vec<Spark>,
}

#[derive(Clone)]
pub struct EnemyDeathSystem {
    store: Rc<EStore>,
    queue: Rc<RefCell<Vec<EnemyDeathEvent>>>,
    active: Rc<RefCell<Vec<DeathFx>>>,
    material: Option<Material>,
    _subs: Rc<SubCollection>,
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
        let material = load_evaporate_material(assets);
        Self {
            store,
            queue,
            active: Rc::new(RefCell::new(Vec::new())),
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
                        sparks: Vec::new(),
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
                        sparks: Vec::new(),
                    }
                }
            };
            (ram_ref, fx)
        };
        self.store.remove(&[ram_ref]);
        self.spawn_sparks(&mut fx);
        self.active.borrow_mut().push(fx);
    }

    fn spawn_sparks(&self, fx: &mut DeathFx) {
        for i in 0..EMBER_COUNT + WISP_COUNT {
            let r0 = gen_range(0.0, 1.0);
            let r1 = gen_range(0.0, 1.0);
            let r2 = gen_range(0.0, 1.0);
            let (vel, size) = if i < EMBER_COUNT {
                // Torn shreds: small random drift, then suction takes over.
                (
                    vec2((r0 - 0.5) * 14.0, (r1 - 0.5) * 10.0),
                    0.8 + r2 * 1.6,
                )
            } else {
                // Void motes: larger, darker, slower.
                (
                    vec2((r0 - 0.5) * 8.0, (r1 - 0.5) * 6.0),
                    1.2 + r2 * 2.0,
                )
            };
            let pos = fx.center + vec2((r0 - 0.5) * fx.dest_size.x, (r1 - 0.5) * fx.dest_size.y);
            fx.sparks.push(Spark { pos, vel, size });
        }
    }
}

// Same sprite-processing material pipeline as the knight afterimage.
fn load_evaporate_material(assets: &Assets) -> Option<Material> {
    let vertex = assets.shader(shader::Shader::DashFxVert);
    let fragment = assets.shader(shader::Shader::EvaporateFrag);

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
                UniformDesc::new("progress", UniformType::Float1),
                UniformDesc::new("edge", UniformType::Float1),
                UniformDesc::new("time", UniformType::Float1),
                UniformDesc::new("ripple", UniformType::Float1),
            ],
            ..Default::default()
        },
    ) {
        Ok(material) => Some(material),
        Err(err) => {
            warn!("evaporate shader failed to compile: {}", err);
            None
        }
    }
}

impl Update for EnemyDeathSystem {
    fn update(&mut self, ctx: &mut Context) {
        let dt = ctx.dt;
        let events: Vec<EnemyDeathEvent> = self.queue.borrow_mut().drain(..).collect();
        for event in events {
            self.begin_fx(event.enemy);
        }
        for fx in self.active.borrow_mut().iter_mut() {
            fx.age += dt;
            for spark in fx.sparks.iter_mut() {
                // Debris is sucked back into the rift: pulled toward the
                // center, damped, and shrinking as it crosses over.
                let inward = fx.center - spark.pos;
                spark.vel += inward * 2.5 * dt;
                spark.vel *= (1.0 - 1.2 * dt).max(0.0);
                spark.pos += spark.vel * dt;
                spark.size = (spark.size - dt * 1.5).max(0.0);
            }
        }
        self.active.borrow_mut().retain(|fx| fx.age < DURATION);
    }
}

impl Draw for EnemyDeathSystem {
    fn draw(&self, _ctx: &Context) {
        for fx in self.active.borrow().iter() {
            let t = (fx.age / DURATION).clamp(0.0, 1.0);
            let pull = PULL * t * t;

            if let Some(material) = self.material.as_ref() {
                gl_use_material(material);
                material.set_uniform(
                    "tint",
                    vec4(VOID_PURPLE.r, VOID_PURPLE.g, VOID_PURPLE.b, 1.0),
                );
                material.set_uniform("progress", t);
                material.set_uniform("edge", EDGE);
                material.set_uniform("time", fx.age);
                material.set_uniform("ripple", 0.6f32);
                draw_texture_ex(
                    &fx.texture,
                    fx.position.x,
                    fx.position.y + pull,
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

            for spark in fx.sparks.iter() {
                let alpha = (1.0 - t).powf(1.2);
                if spark.size > 2.0 {
                    draw_circle(
                        spark.pos.x,
                        spark.pos.y,
                        spark.size,
                        Color::new(VOID_DEEP.r, VOID_DEEP.g, VOID_DEEP.b, alpha * 0.5),
                    );
                } else {
                    draw_circle(
                        spark.pos.x,
                        spark.pos.y,
                        spark.size,
                        Color::new(
                            VOID_PURPLE.r,
                            VOID_PURPLE.g,
                            VOID_PURPLE.b,
                            alpha * 0.7,
                        ),
                    );
                }
            }
        }
    }
}
