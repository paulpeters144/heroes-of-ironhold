use crate::scene::scene_one::components::{Anim, AnimFrame, Blink, Enemy, Physics, Player};
use crate::systems::Update;
use crate::{Context, EStore};
use macroquad::prelude::*;

pub struct HurtSys {
    store: &'static EStore,
}

impl HurtSys {
    pub fn new(store: &'static EStore) -> Self {
        Self { store }
    }
}

fn overlap(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && a.x + a.w > b.x && a.y < b.y + b.h && a.y + a.h > b.y
}

impl Update for HurtSys {
    fn update(&mut self, ctx: &mut Context) {
        let Some(player) = self.store.first::<Player>() else {
            return;
        };
        let hurt = player.hurt;
        let hurt_done = self
            .store
            .get_child::<AnimFrame>(&player)
            .map(|f| f.done)
            .unwrap_or(false);
        let blink_active = self
            .store
            .get_child::<Blink>(&player)
            .map(|b| b.active)
            .unwrap_or(false);

        let mut hit = false;
        if !hurt && !blink_active {
            if let Some(phys) = self.store.get_child::<Physics>(&player) {
                let player_hb = phys.hurtbox(player.pos);
                for id in self.store.ids::<Enemy>() {
                    let Some(enemy) = self.store.get_by_id::<Enemy>(id) else {
                        continue;
                    };
                    let Some(enemy_phys) = self.store.get_child::<Physics>(&enemy) else {
                        continue;
                    };
                    if overlap(player_hb, enemy_phys.hitbox(enemy.pos)) {
                        hit = true;
                        break;
                    }
                }
            }
        }
        drop(player);

        if hit {
            let Some(mut player) = self.store.first_mut::<Player>() else {
                return;
            };
            player.hurt = true;
            player.anim = Anim::Hurt;
        }

        if hurt && hurt_done {
            if let Some(mut player) = self.store.first_mut::<Player>() {
                player.hurt = false;
            }
            if let Some(player) = self.store.first::<Player>() {
                if let Some(mut blink) = self.store.get_child_mut::<Blink>(player) {
                    blink.active = true;
                    blink.timer = 0.0;
                    blink.blinks_left = blink.blink_count;
                    blink.visible = false;
                }
            }
        }

        if blink_active {
            let Some(player) = self.store.first::<Player>() else {
                return;
            };
            let Some(mut blink) = self.store.get_child_mut::<Blink>(player) else {
                return;
            };
            blink.timer += ctx.dt;
            if blink.timer >= blink.blink_interval {
                blink.timer = 0.0;
                blink.visible = !blink.visible;
                if blink.visible {
                    blink.blinks_left -= 1;
                    if blink.blinks_left == 0 {
                        blink.active = false;
                        blink.visible = true;
                    }
                }
            }
        }
    }
}
