use crate::entity::knight::{
    Effect, EffectKind, Knight, Sword, GLOW_IN_FRACTION, HOLD_START, SLASH_LIFETIME,
    SLASH_OFFSET_X, SLASH_OFFSET_Y, SWIPE_FRAME, THRUST_FRAME, THRUST_LIFETIME, THRUST_OFFSET_X,
};
use crate::systems::{Draw, Update};
use crate::{Animation, Context, EStore};
use macroquad::prelude::*;
use pico_entity_store::entity_ref::EntityRef;

pub struct AttackEffectSystem {
    store: &'static EStore,
    prev_frame: Option<usize>,
}

impl AttackEffectSystem {
    pub fn new(store: &'static EStore) -> Self {
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

    fn locate_sword(&self) -> Option<(Vec2, Vec2)> {
        for sword in self.store.all::<Sword>() {
            if let Some(animation) = self.store.get_child::<Animation>(&sword) {
                let size = if animation.dest_size == Vec2::ZERO {
                    vec2(animation.frame_width, animation.frame_height)
                } else {
                    animation.dest_size
                };
                return Some((animation.position, size));
            }
        }
        None
    }

    fn sword_tip(position: Vec2, size: Vec2) -> Vec2 {
        position + vec2(size.x, size.y * 0.5)
    }

    fn spawn(&self, kind: EffectKind, origin: Vec2, lifetime: f32) {
        self.store.add(
            Effect {
                kind,
                age: 0.0,
                lifetime,
                origin,
            },
            &[],
        );
    }
}

impl Update for AttackEffectSystem {
    fn update(&mut self, ctx: &mut Context) {
        let Some(frame) = self.locate_knight_frame() else {
            return;
        };

        if self.prev_frame != Some(frame) {
            self.prev_frame = Some(frame);
            match frame {
                THRUST_FRAME | SWIPE_FRAME => {
                    if let Some((position, size)) = self.locate_sword() {
                        let tip = Self::sword_tip(position, size);
                        let (kind, lifetime) = match frame {
                            THRUST_FRAME => (EffectKind::Thrust, THRUST_LIFETIME),
                            SWIPE_FRAME => (EffectKind::Slash, SLASH_LIFETIME),
                            _ => unreachable!(),
                        };
                        self.spawn(kind, tip, lifetime);
                    }
                }
                _ => {}
            }
        }

        let mut expired: Vec<EntityRef> = Vec::new();
        for mut effect in self.store.all_mut::<Effect>() {
            effect.age += ctx.dt;
            if effect.age >= effect.lifetime {
                expired.push(effect.entity_ref());
            }
        }
        if !expired.is_empty() {
            self.store.remove(&expired);
        }
    }
}

pub struct AttackEffectDrawSystem {
    store: &'static EStore,
    thrust_texture: Texture2D,
    swipe_texture: Texture2D,
}

impl AttackEffectDrawSystem {
    pub fn new(
        store: &'static EStore,
        thrust_texture: Texture2D,
        swipe_texture: Texture2D,
    ) -> Self {
        Self {
            store,
            thrust_texture,
            swipe_texture,
        }
    }
}

fn glow_envelope(t: f32) -> f32 {
    let fade_in = (t / GLOW_IN_FRACTION).clamp(0.0, 1.0).powi(2);
    let fade_out = ((t - HOLD_START) / (1.0 - HOLD_START)).clamp(0.0, 1.0);
    let fade_out = (1.0 - fade_out).powi(2);
    fade_in * fade_out
}

fn draw_texture(texture: &Texture2D, x: f32, y: f32, alpha: f32) {
    let w = texture.width();
    let h = texture.height();
    draw_texture_ex(
        texture,
        x,
        y,
        WHITE.with_alpha(alpha),
        DrawTextureParams {
            dest_size: Some(vec2(w, h)),
            ..Default::default()
        },
    );
}

impl Draw for AttackEffectDrawSystem {
    fn draw(&self, _ctx: &Context) {
        for effect in self.store.all::<Effect>() {
            let t = (effect.age / effect.lifetime).clamp(0.0, 1.0);
            let alpha = glow_envelope(t);

            match effect.kind {
                EffectKind::Thrust => {
                    let x = effect.origin.x + THRUST_OFFSET_X;
                    let y = effect.origin.y - self.thrust_texture.height() * 0.5;
                    draw_texture(&self.thrust_texture, x, y, alpha);
                }
                EffectKind::Slash => {
                    let x = effect.origin.x + SLASH_OFFSET_X - self.swipe_texture.width() * 0.5;
                    let y = effect.origin.y + SLASH_OFFSET_Y - self.swipe_texture.height() * 0.5;
                    draw_texture(&self.swipe_texture, x, y, alpha);
                }
            }
        }
    }
}
