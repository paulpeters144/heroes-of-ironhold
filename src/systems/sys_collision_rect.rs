use crate::prelude::*;
use macroquad::prelude::{vec2, Vec2};
use pico_entity_store::entity_ref::EntityRef;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Copy)]
struct Body {
    anim_ref: EntityRef,
    center: Vec2,
    body_half: Vec2,
}

#[derive(Clone, Copy)]
struct Collider {
    rect_ref: EntityRef,
    center: Vec2,
    rect_half: Vec2,
    body: Option<Body>,
    parent_id: Option<u64>,
}

fn overlaps_at(center: Vec2, half: Vec2, other: &Collider) -> bool {
    (center.x - other.center.x).abs() < half.x + other.rect_half.x
        && (center.y - other.center.y).abs() < half.y + other.rect_half.y
}

pub struct CollisionRectSystem {
    store: Rc<EStore>,
    prev: HashMap<u64, Vec2>,
}

impl CollisionRectSystem {
    pub fn new(store: Rc<EStore>) -> Self {
        Self {
            store,
            prev: HashMap::new(),
        }
    }

    fn collect_bodies(&self) -> HashMap<u64, Body> {
        let mut bodies = HashMap::new();
        for anim in self.store.all::<Animation>() {
            let Some(parent) = self.store.parent(&anim) else {
                continue;
            };
            let r = anim.rect();
            bodies.insert(
                parent.id(),
                Body {
                    anim_ref: anim.entity_ref(),
                    center: vec2(r.x + r.w / 2.0, r.y + r.h / 2.0),
                    body_half: vec2(r.w / 2.0, r.h / 2.0),
                },
            );
        }
        bodies
    }

    fn collect_colliders(&self, bodies: &HashMap<u64, Body>) -> Vec<Collider> {
        let mut colliders = Vec::new();
        for rect in self.store.all::<CollisionRect>() {
            let r = rect.rect;
            let parent_id = self.store.parent(&rect).map(|p| p.id());
            colliders.push(Collider {
                rect_ref: rect.entity_ref(),
                center: vec2(r.x + r.w / 2.0, r.y + r.h / 2.0),
                rect_half: vec2(r.w / 2.0, r.h / 2.0),
                body: parent_id.and_then(|pid| bodies.get(&pid).copied()),
                parent_id,
            });
        }
        colliders
    }
}

impl System for CollisionRectSystem {
    fn update(&mut self, _ctx: &mut Context) {
        let bodies = self.collect_bodies();
        let mut colliders = self.collect_colliders(&bodies);

        for c in &mut colliders {
            if let Some(body) = c.body {
                c.center = body.center;
            }
        }

        let mut next_prev: HashMap<u64, Vec2> = HashMap::new();
        for (idx, c) in colliders.iter().enumerate() {
            let Some(pid) = c.parent_id else {
                continue;
            };
            let Some(body) = c.body else {
                continue;
            };

            let target = body.center;
            let prev = self.prev.get(&pid).copied().unwrap_or(target);

            let blocked_x = colliders.iter().enumerate().any(|(j, other)| {
                j != idx && overlaps_at(vec2(target.x, prev.y), c.rect_half, other)
            });
            let cx = if blocked_x { prev.x } else { target.x };

            let blocked_y = colliders
                .iter()
                .enumerate()
                .any(|(j, other)| j != idx && overlaps_at(vec2(cx, target.y), c.rect_half, other));
            let cy = if blocked_y { prev.y } else { target.y };

            let center = vec2(cx, cy);
            next_prev.insert(pid, center);

            let pos = vec2(center.x - body.body_half.x, center.y - body.body_half.y);
            self.store
                .update::<Animation, _>(&body.anim_ref, |a| a.position = pos);
            self.store.update::<CollisionRect, _>(&c.rect_ref, |r| {
                r.rect.x = center.x - c.rect_half.x;
                r.rect.y = center.y - c.rect_half.y;
            });
        }
        self.prev = next_prev;
    }
}
