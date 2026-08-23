use crate::scene::scene_one::components::{Enemy, EnemyAnim, Patrol, Physics};
use crate::systems::Update;
use crate::{Context, EStore};
use macroquad::prelude::*;

pub struct EnemySys {
    store: &'static EStore,
    colliders: Vec<Rect>,
}

impl EnemySys {
    pub fn new(store: &'static EStore, colliders: Vec<Rect>) -> Self {
        Self { store, colliders }
    }
}

fn overlap(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && a.x + a.w > b.x && a.y < b.y + b.h && a.y + a.h > b.y
}

impl Update for EnemySys {
    fn update(&mut self, ctx: &mut Context) {
        let dt = ctx.dt;
        let ids = self.store.ids::<Enemy>();
        for id in ids {
            let Some(enemy) = self.store.get_by_id::<Enemy>(id) else {
                continue;
            };

            let mut pos = enemy.pos;
            let mut vel = enemy.vel;
            let mut facing = enemy.facing;

            let children = self.store.children(&enemy);
            let mut phys: Option<Physics> = None;
            let mut patrol_id: Option<u64> = None;
            let mut patrol: Option<Patrol> = None;
            for child in &children {
                if let Some(p) = self.store.get_by_id::<Physics>(child.id()) {
                    phys = Some(p.clone());
                }
                if let Some(p) = self.store.get_by_id::<Patrol>(child.id()) {
                    patrol_id = Some(child.id());
                    patrol = Some(p.clone());
                }
            }
            drop(enemy);

            let Some(phys) = phys else {
                continue;
            };

            vel.y += phys.gravity * dt;

            let mut vel_x = 0.0f32;
            if let Some(mut p) = patrol {
                if p.idle_timer > 0.0 {
                    p.idle_timer -= dt;
                } else {
                    let center_x = pos.x + phys.sprite_size * 0.5;
                    let dx = p.target_x - center_x;
                    if dx.abs() < p.reach_dist {
                        pos.x = p.target_x - phys.sprite_size * 0.5;
                        p.idle_timer = p.idle_time;
                        p.target_x = if p.target_x <= p.a + 0.5 { p.b } else { p.a };
                    } else {
                        vel_x = if dx > 0.0 {
                            phys.move_speed
                        } else {
                            -phys.move_speed
                        };
                    }
                }
                patrol = Some(p);
            }
            vel.x = vel_x;

            if vel_x > 0.0 {
                facing = 1.0;
            } else if vel_x < 0.0 {
                facing = -1.0;
            }

            // Horizontal integration and collision.
            pos.x += vel.x * dt;
            let mut hb = phys.hitbox(pos);
            let mut hit_wall = false;
            for r in &self.colliders {
                if overlap(hb, *r) {
                    if vel.x > 0.0 {
                        pos.x = r.x - phys.hitbox_off_x() - phys.hitbox_w;
                    } else if vel.x < 0.0 {
                        pos.x = r.x + r.w - phys.hitbox_off_x();
                    }
                    vel.x = 0.0;
                    hit_wall = true;
                    hb = phys.hitbox(pos);
                }
            }
            if hit_wall {
                if let Some(mut p) = patrol.take() {
                    let center_x = pos.x + phys.sprite_size * 0.5;
                    p.target_x = if center_x <= (p.a + p.b) * 0.5 { p.b } else { p.a };
                    patrol = Some(p);
                }
            }

            // Vertical integration and collision.
            pos.y += vel.y * dt;
            hb = phys.hitbox(pos);
            let mut grounded = false;
            for r in &self.colliders {
                if overlap(hb, *r) {
                    if vel.y > 0.0 {
                        pos.y = r.y - phys.sprite_size;
                        grounded = true;
                    } else if vel.y < 0.0 {
                        pos.y = r.y + r.h - phys.hitbox_off_y();
                    }
                    vel.y = 0.0;
                    hb = phys.hitbox(pos);
                }
            }

            let anim = if !grounded {
                EnemyAnim::Fall
            } else if vel_x != 0.0 {
                EnemyAnim::Walk
            } else {
                EnemyAnim::Idle
            };

            if let Some(mut e) = self.store.get_by_id_mut::<Enemy>(id) {
                e.pos = pos;
                e.vel = vel;
                e.facing = facing;
                e.grounded = grounded;
                e.anim = anim;
            }
            if let (Some(pid), Some(p)) = (patrol_id, patrol) {
                if let Some(mut p_ref) = self.store.get_by_id_mut::<Patrol>(pid) {
                    *p_ref = p;
                }
            }
        }
    }
}
