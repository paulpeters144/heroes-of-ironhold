use crate::entity::knight::{
    Effect, EffectKind, Facing, Knight, Sword, SWIPE_FRAME, THRUST_FRAME,
};
use crate::entity::player::PlayerOne;
use crate::systems::{Draw, Update};
// use crate::util::{did_attack, ImageData};
use crate::{Animation, AttackRect, Context, EStore};
use macroquad::prelude::*;
use std::rc::Rc;

const ATTACK_SIZE_FRACTION: f32 = 0.75;

#[derive(Clone)]
pub struct KnightAttackSystem {
    store: Rc<EStore>,
}

impl KnightAttackSystem {
    pub fn new(store: Rc<EStore>) -> Self {
        Self { store }
    }
}

impl Update for KnightAttackSystem {
    fn update(&mut self, _ctx: &mut Context) {
        let Some(attack_ref) = self
            .store
            .first::<PlayerOne>()
            .and_then(|p| self.store.get_child::<Knight>(&p))
            .and_then(|k| self.store.get_by_id::<Knight>(k.id()))
            .and_then(|k| self.store.get_child::<Sword>(&k))
            .and_then(|s| self.store.get_child::<AttackRect>(&s))
            .map(|a| a.entity_ref())
        else {
            return;
        };

        let (sword_rect, sword_pos, sword_visible) = {
            let Some(sword) = self
                .store
                .first::<PlayerOne>()
                .and_then(|p| self.store.get_child::<Knight>(&p))
                .and_then(|k| self.store.get_by_id::<Knight>(k.id()))
                .and_then(|k| self.store.get_child::<Sword>(&k))
                .and_then(|s| self.store.get_child::<Animation>(&s))
            else {
                return;
            };
            (sword.rect(), sword.position, sword.visible)
        };

        let Some(frame) = self
            .store
            .first::<PlayerOne>()
            .and_then(|p| self.store.get_child::<Knight>(&p))
            .and_then(|k| self.store.get_by_id::<Knight>(k.id()))
            .and_then(|k| self.store.get_child::<Animation>(&k))
            .map(|a| a.current_frame)
        else {
            return;
        };

        let mirror = self
            .store
            .first::<PlayerOne>()
            .and_then(|p| self.store.get_child::<Knight>(&p))
            .and_then(|k| self.store.get_by_id::<Knight>(k.id()))
            .and_then(|k| self.store.get_child::<Facing>(&k))
            .is_some_and(|f| *f == Facing::Left);

        let attack = match frame {
            THRUST_FRAME => Some(EffectKind::Thrust),
            SWIPE_FRAME => Some(EffectKind::Slash),
            _ => None,
        };

        let effect_rect = attack.and_then(|kind| {
            self.store
                .all::<Effect>()
                .find(|e| e.kind == kind)
                .map(|e| {
                    let full_w = e.texture.width();
                    let full_h = e.texture.height();
                    let w = full_w * ATTACK_SIZE_FRACTION;
                    let h = full_h * ATTACK_SIZE_FRACTION;
                    let cx = if mirror {
                        sword_pos.x + sword_rect.w - e.offset.x - full_w / 2.0
                    } else {
                        sword_pos.x + e.offset.x + full_w / 2.0
                    };
                    let cy = sword_pos.y + e.offset.y + full_h / 2.0;
                    Rect::new(cx - w / 2.0, cy - h / 2.0, w, h)
                })
        });

        let mut rects = Vec::with_capacity(2);
        if sword_visible {
            let mut r = sword_rect;
            r.y += r.h * 0.3;
            r.h *= 0.4;
            rects.push(r);
        }
        if let Some(er) = effect_rect {
            rects.push(er);
        }

        let visible = attack.is_some();
        self.store.update::<AttackRect, _>(&attack_ref, |area| {
            area.rects = rects;
            area.visible = visible;
        });
    }
}

impl Draw for KnightAttackSystem {
    fn draw(&self, ctx: &Context) {
        if !ctx.debug {
            return;
        }
        let Some(area) = self
            .store
            .first::<PlayerOne>()
            .and_then(|p| self.store.get_child::<Knight>(&p))
            .and_then(|k| self.store.get_by_id::<Knight>(k.id()))
            .and_then(|k| self.store.get_child::<Sword>(&k))
            .and_then(|s| self.store.get_child::<AttackRect>(&s))
        else {
            return;
        };
        if !area.visible {
            return;
        }
        for rect in &area.rects {
            draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, YELLOW);
        }
    }
}
