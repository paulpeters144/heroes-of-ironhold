use crate::entity::factory_hero::SWORD_FRAME_SIZE;
use crate::entity::knight::{
    Effect, EffectKind, Facing, Knight, KnightLock, Sword, GLOW_IN_FRACTION, HOLD_START,
    SWIPE_FRAME, THRUST_FRAME,
};
use crate::entity::player::PlayerOne;
use crate::systems::System;
use crate::{Animation, Context, EStore};
use macroquad::prelude::*;
use std::rc::Rc;

pub struct KnightAttackEffectSystem {
    store: Rc<EStore>,
    prev_frame: Option<usize>,
}

impl KnightAttackEffectSystem {
    pub fn new(store: Rc<EStore>) -> Self {
        Self {
            store,
            prev_frame: None,
        }
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

impl System for KnightAttackEffectSystem {
    fn update(&mut self, ctx: &mut Context) {
        // Skip the knight while a skill holds it (thrust pose is cosmetic).
        if self
            .store
            .first::<PlayerOne>()
            .and_then(|p| self.store.get_child::<Knight>(&p))
            .and_then(|k| self.store.get_child::<KnightLock>(&k))
            .is_some()
        {
            return;
        }

        let Some(frame) = self
            .store
            .first::<PlayerOne>()
            .and_then(|player| self.store.get_child::<Knight>(&player))
            .and_then(|knight| self.store.get_child::<Animation>(&knight))
            .map(|a| a.current_frame)
        else {
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

    fn draw(&self, _ctx: &Context) {
        let Some(sword_pos) = self
            .store
            .first::<PlayerOne>()
            .and_then(|p| self.store.get_child::<Knight>(&p))
            .and_then(|k| self.store.get_by_id::<Knight>(k.id()))
            .and_then(|k| self.store.get_child::<Sword>(&k))
            .and_then(|s| self.store.get_child::<Animation>(&s))
            .map(|a| a.position)
        else {
            return;
        };

        let mirror = self
            .store
            .first::<PlayerOne>()
            .and_then(|p| self.store.get_child::<Knight>(&p))
            .and_then(|r| self.store.get_by_id::<Knight>(r.id()))
            .and_then(|k| self.store.get_child::<Facing>(&k))
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
