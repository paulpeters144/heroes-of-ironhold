use crate::entity::CollisionGroup;
use crate::prelude::*;
use macroquad::prelude::{vec2, Rect, Vec2};
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
    group: CollisionGroup,
    body: Option<Body>,
    parent_id: Option<u64>,
}

fn circles_overlap(center: Vec2, radius: f32, other: &Collider) -> bool {
    let delta = center - other.center;
    let dist = delta.length();
    dist < radius + other.radius
}

fn groups_interact(a: CollisionGroup, b: CollisionGroup) -> bool {
    !matches!(
        (a, b),
        (CollisionGroup::Hero, CollisionGroup::Peon) | (CollisionGroup::Peon, CollisionGroup::Hero)
    )
}

/// Whether a circle overlaps an axis-aligned rectangle (without moving it).
fn circle_rect_overlap(center: Vec2, radius: f32, rect: &Rect) -> bool {
    let closest_x = center.x.clamp(rect.x, rect.x + rect.w);
    let closest_y = center.y.clamp(rect.y, rect.y + rect.h);
    let dx = center.x - closest_x;
    let dy = center.y - closest_y;
    dx * dx + dy * dy < radius * radius
}

/// Push a circle out of an axis-aligned rectangle. Returns `true` if the circle
/// was moved.
fn resolve_circle_rect(center: &mut Vec2, radius: f32, rect: &Rect) -> bool {
    let closest_x = center.x.clamp(rect.x, rect.x + rect.w);
    let closest_y = center.y.clamp(rect.y, rect.y + rect.h);
    let dx = center.x - closest_x;
    let dy = center.y - closest_y;
    let dist_sq = dx * dx + dy * dy;
    if dist_sq >= radius * radius {
        return false;
    }

    if dist_sq > f32::EPSILON {
        let dist = dist_sq.sqrt();
        let push = radius - dist;
        center.x += dx / dist * push;
        center.y += dy / dist * push;
    } else {
        // Center is inside the rect; push out along the smallest penetration axis.
        let left = center.x - rect.x;
        let right = rect.x + rect.w - center.x;
        let top = center.y - rect.y;
        let bottom = rect.y + rect.h - center.y;
        let min = left.min(right).min(top).min(bottom);
        if min == left {
            center.x = rect.x - radius;
        } else if min == right {
            center.x = rect.x + rect.w + radius;
        } else if min == top {
            center.y = rect.y - radius;
        } else {
            center.y = rect.y + rect.h + radius;
        }
    }
    true
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
                group: circle.group,
                body,
                parent_id,
            });
        }
        colliders
    }

    fn collect_rects(&self) -> Vec<Rect> {
        self.store.all::<CollisionRect>().map(|r| r.rect).collect()
    }
}

impl System for CollisionCircleSystem {
    fn update(&mut self, _ctx: &mut Context) {
        let bodies = self.collect_bodies();
        let mut colliders = self.collect_colliders(&bodies);
        let rects = self.collect_rects();

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
                j != idx
                    && groups_interact(c.group, other.group)
                    && circles_overlap(vec2(target.x, prev.y), c.radius, other)
            }) || rects
                .iter()
                .any(|r| circle_rect_overlap(vec2(target.x, prev.y), c.radius, r));
            let cx = if blocked_x { prev.x } else { target.x };

            let blocked_y = colliders
                .iter()
                .enumerate()
                .any(|(j, other)| {
                    j != idx
                        && groups_interact(c.group, other.group)
                        && circles_overlap(vec2(cx, target.y), c.radius, other)
                })
                || rects
                    .iter()
                    .any(|r| circle_rect_overlap(vec2(cx, target.y), c.radius, r));
            let cy = if blocked_y { prev.y } else { target.y };

            let mut center = vec2(cx, cy);

            for _ in 0..4 {
                let mut pushed = false;
                for (j, other) in colliders.iter().enumerate() {
                    if j == idx || !groups_interact(c.group, other.group) {
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
                for r in &rects {
                    if resolve_circle_rect(&mut center, c.radius, r) {
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
