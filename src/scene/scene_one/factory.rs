use macroquad::prelude::*;

use pico_entity_store::children;
use pico_entity_store::store::ChildSource;

use crate::image;
use crate::scene::scene_one::components::{
    AnimFrame, Enemy, EnemyAnim, EnemySpriteDefs, Patrol, Physics, SpriteDef,
};

pub struct EnemyFactory;

impl EnemyFactory {
    pub fn new() -> Self {
        Self
    }

    pub fn sprite_defs(&self) -> EnemySpriteDefs {
        EnemySpriteDefs {
            idle: SpriteDef {
                image: image::Id::E3,
                first_frame: 0,
                frame_count: 5,
                sheet_frames: 14,
                frame_time: 0.15,
                once: false,
            },
            walk: SpriteDef {
                image: image::Id::E3,
                first_frame: 5,
                frame_count: 8,
                sheet_frames: 14,
                frame_time: 0.1,
                once: false,
            },
            fall: SpriteDef {
                image: image::Id::E3,
                first_frame: 13,
                frame_count: 1,
                sheet_frames: 14,
                frame_time: 0.1,
                once: true,
            },
        }
    }

    pub fn build_enemy_one(&self, patrol: [Vec2; 2]) -> (Enemy, Vec<ChildSource>) {
        let (a, b) = if patrol[0].x <= patrol[1].x {
            (patrol[0].x, patrol[1].x)
        } else {
            (patrol[1].x, patrol[0].x)
        };

        let phys = Physics {
            move_speed: 40.0,
            gravity: 900.0,
            jump_velocity: -320.0,
            sprite_size: 32.0,
            hitbox_w: 20.0,
            hitbox_h: 30.0,
        };

        let enemy = Enemy {
            pos: Vec2::new(patrol[0].x - phys.sprite_size * 0.5, patrol[0].y),
            vel: Vec2::ZERO,
            facing: -1.0,
            grounded: false,
            anim: EnemyAnim::Fall,
        };

        let children = Vec::from(children![
            phys,
            AnimFrame {
                frame: 0,
                elapsed: 0.0,
                idle_delay: 0.0,
                done: false,
            },
            Patrol {
                a,
                b,
                target_x: a,
                idle_timer: 0.0,
                reach_dist: 2.0,
                idle_time: 1.2,
            },
        ]);

        (enemy, children)
    }
}
