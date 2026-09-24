use crate::entity::{AreaRect, PeonSpawnZone, RamHeadSpawnZone};
use crate::prelude::*;
use macroquad::prelude::{
    draw_circle_lines, draw_rectangle_lines, vec2, Color, BLUE, GREEN, ORANGE, SKYBLUE, YELLOW,
};
use std::collections::HashMap;
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

        for zone in self.store.all::<PeonSpawnZone>() {
            draw_rectangle_lines(zone.rect.x, zone.rect.y, zone.rect.w, zone.rect.h, 2.0, SKYBLUE);
        }

        for zone in self.store.all::<RamHeadSpawnZone>() {
            draw_rectangle_lines(zone.rect.x, zone.rect.y, zone.rect.w, zone.rect.h, 2.0, ORANGE);
        }

        for area in self.store.all::<AttackRect>() {
            if !area.visible {
                continue;
            }
            for rect in &area.rects {
                draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, YELLOW);
            }
        }

        let mut body_centers = HashMap::new();
        for anim in self.store.all::<Animation>() {
            if let Some(parent) = self.store.parent(&anim) {
                let r = anim.rect();
                body_centers.insert(parent.id(), vec2(r.x + r.w / 2.0, r.y + r.h / 2.0));
            }
        }

        for circle in self.store.all::<CollisionCircle>() {
            let parent_id = self.store.parent(&circle).map(|p| p.id());
            let dynamic = parent_id.is_some_and(|id| body_centers.contains_key(&id));
            let color = if dynamic {
                Color::new(1.0, 0.0, 0.0, 1.0)
            } else {
                Color::new(0.0, 0.0, 1.0, 1.0)
            };
            let center = parent_id
                .and_then(|id| body_centers.get(&id).copied())
                .unwrap_or(circle.center);
            draw_circle_lines(center.x, center.y, circle.radius, 2.0, color);
        }

        for rect in self.store.all::<CollisionRect>() {
            draw_rectangle_lines(rect.rect.x, rect.rect.y, rect.rect.w, rect.rect.h, 2.0, BLUE);
        }
    }
}
