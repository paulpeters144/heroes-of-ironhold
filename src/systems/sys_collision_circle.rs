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
    center: Vec2,
    radius: f32,
    body: Option<Body>,
    parent_id: Option<u64>,
}

fn circles_overlap(center: Vec2, radius: f32, other: &Collider) -> bool {
    let delta = center - other.center;
    let dist = delta.length();
    dist < radius + other.radius
}

pub struct CollisionCircleSystem {
    store: Rc<EStore>,
    prev: HashMap<u64, Vec2>,
}

impl CollisionCircleSystem {
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
        for circle in self.store.all::<CollisionCircle>() {
            let parent_id = self.store.parent(&circle).map(|p| p.id());
            let body = parent_id.and_then(|pid| bodies.get(&pid).copied());
            let center = body.map(|b| b.center).unwrap_or(Vec2::ZERO);
            colliders.push(Collider {
                center,
                radius: circle.radius,
                body,
                parent_id,
            });
        }
        colliders
    }
}

impl System for CollisionCircleSystem {
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
                j != idx && circles_overlap(vec2(target.x, prev.y), c.radius, other)
            });
            let cx = if blocked_x { prev.x } else { target.x };

            let blocked_y = colliders
                .iter()
                .enumerate()
                .any(|(j, other)| j != idx && circles_overlap(vec2(cx, target.y), c.radius, other));
            let cy = if blocked_y { prev.y } else { target.y };

            let mut center = vec2(cx, cy);

            for _ in 0..4 {
                let mut pushed = false;
                for (j, other) in colliders.iter().enumerate() {
                    if j == idx {
                        continue;
                    }
                    let delta = center - other.center;
                    let dist = delta.length();
                    let min_dist = c.radius + other.radius;
                    if dist < min_dist {
                        let dir = if dist > 0.001 {
                            delta / dist
                        } else {
                            vec2(1.0, 0.0)
                        };
                        center += dir * (min_dist - dist);
                        pushed = true;
                    }
                }
                if !pushed {
                    break;
                }
            }

            next_prev.insert(pid, center);

            let pos = vec2(center.x - body.body_half.x, center.y - body.body_half.y);
            self.store
                .update::<Animation, _>(&body.anim_ref, |a| a.position = pos);
        }
        self.prev = next_prev;
    }
}
