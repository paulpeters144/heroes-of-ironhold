use crate::input::{self, Input};
use crate::scene::scene_one::components::{Anim, Physics, Player};
use crate::systems::Update;
use crate::{Config, Context, EStore};
use macroquad::prelude::*;

pub struct PlayerSys {
    store: &'static EStore,
    cfg: &'static Config,
    colliders: Vec<Rect>,
    map_size_px: Vec2,
}

impl PlayerSys {
    pub fn new(
        store: &'static EStore,
        cfg: &'static Config,
        colliders: Vec<Rect>,
        map_size_px: Vec2,
    ) -> Self {
        Self {
            store,
            cfg,
            colliders,
            map_size_px,
        }
    }
}

fn overlap(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && a.x + a.w > b.x && a.y < b.y + b.h && a.y + a.h > b.y
}

fn clamp_axis(target: f32, map: f32, view: f32) -> f32 {
    if map <= view {
        -(view - map) * 0.5
    } else {
        target.clamp(0.0, map - view)
    }
}

impl Update for PlayerSys {
    fn update(&mut self, ctx: &mut Context) {
        let dt = ctx.dt;
        let Some(phys) = self
            .store
            .first::<Player>()
            .and_then(|player| self.store.get_child::<Physics>(&player))
            .map(|p| p.clone())
        else {
            return;
        };
        let Some(mut player) = self.store.first_mut::<Player>() else {
            return;
        };

        let hurt = player.hurt;

        let mut dir = 0.0;
        if !hurt {
            if input::down(Input::Left) {
                dir -= 1.0;
            }
            if input::down(Input::Right) {
                dir += 1.0;
            }
            player.vel.x = dir * phys.move_speed;
            if dir > 0.0 {
                player.facing = 1.0;
            } else if dir < 0.0 {
                player.facing = -1.0;
            }

            if input::down_once(Input::Jump) && player.grounded {
                player.vel.y = phys.jump_velocity;
                player.grounded = false;
            }

            player.vel.y += phys.gravity * dt;

            // Horizontal integration and collision.
            player.pos.x += player.vel.x * dt;
            let mut hb = phys.hitbox(player.pos);
            for r in &self.colliders {
                if overlap(hb, *r) {
                    if player.vel.x > 0.0 {
                        player.pos.x = r.x - phys.hitbox_off_x() - phys.hitbox_w;
                    } else if player.vel.x < 0.0 {
                        player.pos.x = r.x + r.w - phys.hitbox_off_x();
                    }
                    player.vel.x = 0.0;
                    hb = phys.hitbox(player.pos);
                }
            }

            // Vertical integration and collision.
            player.pos.y += player.vel.y * dt;
            hb = phys.hitbox(player.pos);
            player.grounded = false;
            for r in &self.colliders {
                if overlap(hb, *r) {
                    if player.vel.y > 0.0 {
                        player.pos.y = r.y - phys.sprite_size;
                        player.grounded = true;
                    } else if player.vel.y < 0.0 {
                        player.pos.y = r.y + r.h - phys.hitbox_off_y();
                    }
                    player.vel.y = 0.0;
                    hb = phys.hitbox(player.pos);
                }
            }
        } else {
            player.vel.x = 0.0;
            player.vel.y = 0.0;
        }

        player.anim = if hurt {
            Anim::Hurt
        } else if !player.grounded {
            Anim::Jump
        } else if dir != 0.0 {
            Anim::Run
        } else {
            Anim::Idle
        };

        // Camera follows the player on both axes, clamped to map bounds.
        ctx.cam_zoom = 1.0;
        let view_w = self.cfg.v_width;
        let view_h = self.cfg.v_height;
        let center = vec2(
            player.pos.x + phys.sprite_size * 0.5,
            player.pos.y + phys.sprite_size * 0.5,
        );
        ctx.cam_pan = vec2(
            clamp_axis(center.x - view_w * 0.5, self.map_size_px.x, view_w),
            clamp_axis(center.y - view_h * 0.5, self.map_size_px.y, view_h),
        );
    }
}
