use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

use crate::entity::enemy::{EnemyStats, RamHead};
use crate::entity::hero::HeroStats;
use crate::entity::knight::Knight;
use crate::events::{AttackEvent, HealthChangeEvent};
use crate::systems::{Draw, Update};
use crate::{shader, Animation, Assets, Context, EStore, EventBus, SubCollection};
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use macroquad::prelude::*;

const FLASH_DURATION: f32 = 0.15;

#[derive(Clone)]
pub struct HandleAttackSystem {
    store: Rc<EStore>,
    bus: Rc<EventBus>,
    flash_material: Rc<Option<Material>>,
    flashing: Rc<RefCell<HashMap<u64, f32>>>,
    queue: Rc<RefCell<VecDeque<AttackEvent>>>,
    _subs: Rc<SubCollection>,
}

impl HandleAttackSystem {
    pub fn new(store: Rc<EStore>, assets: &Assets, bus: Rc<EventBus>) -> Self {
        let queue = Rc::new(RefCell::new(VecDeque::new()));
        let subs = Rc::new(SubCollection::new());
        let queue_for_handler = queue.clone();
        subs.on::<AttackEvent>(&bus, move |event: &AttackEvent| {
            queue_for_handler.borrow_mut().push_back(event.clone());
        });
        Self {
            store,
            bus,
            flash_material: Rc::new(load_flash_material(assets)),
            flashing: Rc::new(RefCell::new(HashMap::new())),
            queue,
            _subs: subs,
        }
    }
}

fn load_flash_material(assets: &Assets) -> Option<Material> {
    let vertex = assets.shader(shader::Shader::DashFxVert);
    let fragment = assets.shader(shader::Shader::FlashWhiteFrag);

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
            ..Default::default()
        },
    ) {
        Ok(material) => Some(material),
        Err(err) => {
            warn!("flash white shader failed to compile: {}", err);
            None
        }
    }
}

impl Update for HandleAttackSystem {
    fn update(&mut self, ctx: &mut Context) {
        while let Some(event) = self.queue.borrow_mut().pop_front() {
            let damage = self
                .store
                .get_by_id::<Knight>(event.attacker)
                .and_then(|knight| self.store.get_child::<HeroStats>(&knight))
                .map(|stats| stats.strength)
                .unwrap_or(1)
                .max(1);

            for target in event.targets {
                self.flashing.borrow_mut().insert(target, FLASH_DURATION);

                let stats_ref = self
                    .store
                    .get_by_id::<RamHead>(target)
                    .and_then(|enemy| self.store.get_child::<EnemyStats>(&enemy))
                    .map(|stats| stats.entity_ref());
                if let Some(stats_ref) = stats_ref {
                    self.store.update::<EnemyStats, _>(&stats_ref, |stats| {
                        stats.hp = (stats.hp - damage).max(0);
                    });

                    let rect = self
                        .store
                        .get_by_id::<RamHead>(target)
                        .and_then(|enemy| self.store.get_child::<Animation>(&enemy))
                        .map(|anim| anim.rect());
                    if let Some(rect) = rect {
                        self.bus.fire(&HealthChangeEvent {
                            entity: target,
                            amount: -(damage),
                            rect,
                        });
                    }
                }
            }
        }

        let dt = ctx.dt;
        self.flashing.borrow_mut().retain(|_, remaining| {
            *remaining -= dt;
            *remaining > 0.0
        });
    }
}

impl Draw for HandleAttackSystem {
    fn draw(&self, _ctx: &Context) {
        if let Some(material) = self.flash_material.as_ref() {
            gl_use_material(material);
            for (id, remaining) in self.flashing.borrow().iter() {
                if *remaining <= 0.0 {
                    continue;
                }
                let Some(animation) = self
                    .store
                    .get_by_id::<RamHead>(*id)
                    .and_then(|e| self.store.get_child::<Animation>(&e))
                else {
                    continue;
                };

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

                draw_texture_ex(
                    &animation.source,
                    animation.position.x,
                    animation.position.y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(dest_size),
                        source: Some(source),
                        flip_x: animation.flip_x,
                        ..Default::default()
                    },
                );
            }
            gl_use_default_material();
        }
    }
}
