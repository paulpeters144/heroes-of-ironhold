use crate::entity::enemy::{
    RamHead, ATTACK_FRAMES, ATTACK_RANGE, FRAME_DURATION, IDLE_FRAME, MOVE_SPEED,
    MOVE_SPEED_VERTICAL, WALK_FRAMES,
};
use crate::entity::knight::Knight;
use crate::entity::player::PlayerOne;
use crate::systems::Update;
use crate::{Animation, Context, EStore};
use macroquad::prelude::vec2;
use std::rc::Rc;

pub struct EnemyAiSystem {
    store: Rc<EStore>,
    frame_elapsed: f32,
    walk_step: usize,
    attack_step: usize,
    attacking: bool,
}

impl EnemyAiSystem {
    pub fn new(store: Rc<EStore>) -> Self {
        Self {
            store,
            frame_elapsed: 0.0,
            walk_step: 0,
            attack_step: 0,
            attacking: false,
        }
    }

    fn next_frame(&mut self, dt: f32) -> usize {
        self.frame_elapsed += dt;
        if self.frame_elapsed < FRAME_DURATION {
            return if self.attacking {
                ATTACK_FRAMES[self.attack_step]
            } else {
                WALK_FRAMES[self.walk_step]
            };
        }
        self.frame_elapsed = 0.0;

        if self.attacking {
            self.attack_step = (self.attack_step + 1) % ATTACK_FRAMES.len();
            ATTACK_FRAMES[self.attack_step]
        } else {
            self.walk_step = (self.walk_step + 1) % WALK_FRAMES.len();
            WALK_FRAMES[self.walk_step]
        }
    }
}

impl Update for EnemyAiSystem {
    fn update(&mut self, ctx: &mut Context) {
        let Some(anim_ref) = self
            .store
            .first::<RamHead>()
            .and_then(|enemy| self.store.get_child::<Animation>(&enemy))
            .map(|a| a.entity_ref())
        else {
            return;
        };

        let Some(enemy_pos) = self
            .store
            .get_by_id::<Animation>(anim_ref.id())
            .map(|a| a.position)
        else {
            return;
        };

        let Some(knight_pos) = self
            .store
            .first::<PlayerOne>()
            .and_then(|p| self.store.get_child::<Knight>(&p))
            .and_then(|k| self.store.get_child::<Animation>(&k))
            .map(|a| a.position)
        else {
            self.store.update::<Animation, _>(&anim_ref, |animation| {
                animation.current_frame = IDLE_FRAME;
            });
            return;
        };

        let delta = knight_pos - enemy_pos;
        let distance = delta.length();
        let in_range = distance <= ATTACK_RANGE;

        if in_range != self.attacking {
            self.attacking = in_range;
            self.frame_elapsed = 0.0;
            self.walk_step = 0;
            self.attack_step = 0;
        }

        let mut new_pos = enemy_pos;
        if !self.attacking {
            let dir = delta / distance;
            new_pos += vec2(dir.x * MOVE_SPEED, dir.y * MOVE_SPEED_VERTICAL) * ctx.dt;
        }

        let new_frame = self.next_frame(ctx.dt);
        let flip_x = delta.x < 0.0;

        self.store.update::<Animation, _>(&anim_ref, |animation| {
            animation.position.x = new_pos.x.round();
            animation.position.y = new_pos.y.round();
            animation.current_frame = new_frame;
            animation.flip_x = flip_x;
        });
    }
}
