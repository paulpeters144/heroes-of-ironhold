use crate::systems::System;
use crate::{Animation, AreaRect, AttackRect, CollisionRect, Context, EStore};
use macroquad::prelude::{draw_rectangle_lines, Color, GREEN, YELLOW};
use std::collections::HashSet;
use std::rc::Rc;

pub struct DebugDrawSystem {
    store: Rc<EStore>,
}

impl DebugDrawSystem {
    pub fn new(store: Rc<EStore>) -> Self {
        Self { store }
    }
}

impl System for DebugDrawSystem {
    fn draw(&self, ctx: &Context) {
        if !ctx.debug {
            return;
        }

        for area in self.store.all::<AreaRect>() {
            draw_rectangle_lines(
                area.rect.x,
                area.rect.y,
                area.rect.w,
                area.rect.h,
                2.0,
                GREEN,
            );
        }

        for area in self.store.all::<AttackRect>() {
            if !area.visible {
                continue;
            }
            for rect in &area.rects {
                draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, YELLOW);
            }
        }

        let mut body_parents = HashSet::new();
        for anim in self.store.all::<Animation>() {
            if let Some(parent) = self.store.parent(&anim) {
                body_parents.insert(parent.id());
            }
        }

        for rect in self.store.all::<CollisionRect>() {
            let dynamic = self
                .store
                .parent(&rect)
                .is_some_and(|p| body_parents.contains(&p.id()));
            let color = if dynamic {
                Color::new(1.0, 0.0, 0.0, 1.0)
            } else {
                Color::new(0.0, 0.0, 1.0, 1.0)
            };
            draw_rectangle_lines(
                rect.rect.x,
                rect.rect.y,
                rect.rect.w,
                rect.rect.h,
                2.0,
                color,
            );
        }
    }
}
