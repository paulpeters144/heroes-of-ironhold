use crate::entity::factory_hero::SWORD_FRAME_SIZE;
use crate::entity::knight::{
    Effect, EffectKind, Facing, Knight, Sword, GLOW_IN_FRACTION, HOLD_START, SWIPE_FRAME,
    THRUST_FRAME,
};
use crate::systems::{Draw, Update};
use crate::{Animation, Context, EStore};
use macroquad::prelude::*;
use std::rc::Rc;

pub struct AttackEffectSystem {
    store: Rc<EStore>,
    prev_frame: Option<usize>,
}

impl AttackEffectSystem {
    pub fn new(store: Rc<EStore>) -> Self {
        Self {
            store,
            prev_frame: None,
        }
    }

    fn locate_knight_frame(&self) -> Option<usize> {
        for knight in self.store.all::<Knight>() {
            if let Some(animation) = self.store.get_child::<Animation>(&knight) {
                return Some(animation.current_frame);
            }
        }
        None
    }
}

impl Update for AttackEffectSystem {
    fn update(&mut self, ctx: &mut Context) {
        let Some(frame) = self.locate_knight_frame() else {
            return;
        };

        let triggered = if self.prev_frame == Some(frame) {
            None
        } else {
            self.prev_frame = Some(frame);
            match frame {
                THRUST_FRAME => Some(EffectKind::Thrust),
                SWIPE_FRAME => Some(EffectKind::Slash),
                _ => None,
            }
        };

        for mut effect in self.store.all_mut::<Effect>() {
            if triggered == Some(effect.kind) {
                effect.visible = true;
                effect.age = 0.0;
            }
            if effect.visible {
                effect.age += ctx.dt;
                if effect.age >= effect.lifetime {
                    effect.visible = false;
                }
            }
        }
    }
}

pub struct AttackEffectDrawSystem {
    store: Rc<EStore>,
}

impl AttackEffectDrawSystem {
    pub fn new(store: Rc<EStore>) -> Self {
        Self { store }
    }

    fn sword_position(&self) -> Option<Vec2> {
        let sword = self.store.first::<Sword>()?;
        let animation = self.store.get_child::<Animation>(&sword)?;
        Some(animation.position)
    }
}

fn glow_envelope(t: f32) -> f32 {
    let fade_in = (t / GLOW_IN_FRACTION).clamp(0.0, 1.0).powi(2);
    let fade_out = ((t - HOLD_START) / (1.0 - HOLD_START)).clamp(0.0, 1.0);
    let fade_out = (1.0 - fade_out).powi(2);
    fade_in * fade_out
}

fn draw_texture(texture: &Texture2D, x: f32, y: f32, alpha: f32, flip_x: bool) {
    let w = texture.width();
    let h = texture.height();
    draw_texture_ex(
        texture,
        x,
        y,
        WHITE.with_alpha(alpha),
        DrawTextureParams {
            dest_size: Some(vec2(w, h)),
            flip_x,
            ..Default::default()
        },
    );
}

impl Draw for AttackEffectDrawSystem {
    fn draw(&self, _ctx: &Context) {
        let Some(sword_pos) = self.sword_position() else {
            return;
        };

        let mirror = self
            .store
            .first::<Facing>()
            .is_some_and(|f| *f == Facing::Left);

        for effect in self.store.all::<Effect>() {
            if !effect.visible {
                continue;
            }
            let t = (effect.age / effect.lifetime).clamp(0.0, 1.0);
            let alpha = glow_envelope(t);
            let (x, flip_x) = if mirror {
                let w = effect.texture.width();
                (sword_pos.x + SWORD_FRAME_SIZE - effect.offset.x - w, true)
            } else {
                (sword_pos.x + effect.offset.x, false)
            };
            let y = sword_pos.y + effect.offset.y;
            draw_texture(&effect.texture, x, y, alpha, flip_x);
        }
    }
}
