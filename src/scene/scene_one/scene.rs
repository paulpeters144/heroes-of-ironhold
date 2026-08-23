use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use macroquad::prelude::*;
use tiled::{TiledMap, TiledMapCfg};

use pico_entity_store::children;

use crate::scene::scene_one::components::{
    AnimFrame, Blink, Physics, Player, PlayerSpriteDefs, SpriteDef,
};
use crate::scene::scene_one::factory::EnemyFactory;
use crate::scene::scene_one::sys_animation::AnimationSys;
use crate::scene::scene_one::sys_draw::DrawSys;
use crate::scene::scene_one::sys_enemy::EnemySys;
use crate::scene::scene_one::sys_enemy_animation::EnemyAnimationSys;
use crate::scene::scene_one::sys_hurt::HurtSys;
use crate::scene::scene_one::sys_player::PlayerSys;
use crate::scene::Scene;
use crate::systems::SystemAgg;
use crate::{file, image, Assets, Config, Context, EStore};

pub struct SceneOne {
    cfg: &'static Config,
    assets: &'static Assets,
    estore: &'static EStore,
    sys: &'static SystemAgg,
}

impl SceneOne {
    pub fn new(
        cfg: &'static Config,
        assets: &'static Assets,
        estore: &'static EStore,
        system_agg: &'static SystemAgg,
    ) -> Self {
        Self {
            cfg,
            assets,
            estore,
            sys: system_agg,
        }
    }
}

impl Scene for SceneOne {
    fn load(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        self.sys.clear();
        self.estore.clear();

        let tile_map = self
            .assets
            .file(file::Id::Scene1Tmx)
            .cloned()
            .unwrap_or_default();

        let mut tile_sets = HashMap::new();
        if let Some(tsx) = self.assets.file(file::Id::TileSetTsx) {
            tile_sets.insert("tile-set.tsx".to_string(), tsx.clone());
        }

        let map = TiledMap::from_config(TiledMapCfg {
            tile_map,
            tile_sets,
            images: &self.assets.images,
            section_size: (16, 16),
        });

        let (map, colliders, map_size_px) = match map {
            Ok(m) => {
                let colliders = m.collide_statics.iter().map(|cs| cs.0).collect();
                let map_size_px = vec2(
                    m.map_size.0 as f32 * m.tile_size.0 as f32,
                    m.map_size.1 as f32 * m.tile_size.1 as f32,
                );
                (Some(m), colliders, map_size_px)
            }
            Err(err) => {
                warn!("failed to load scene-1 tile map: {:?}", err);
                (None, Vec::new(), vec2(0.0, 0.0))
            }
        };

        let player_defs = PlayerSpriteDefs {
            idle: SpriteDef {
                image: image::Id::MbIdle,
                first_frame: 0,
                frame_count: 5,
                sheet_frames: 5,
                frame_time: 0.15,
                once: false,
            },
            run: SpriteDef {
                image: image::Id::MbRun,
                first_frame: 0,
                frame_count: 8,
                sheet_frames: 8,
                frame_time: 0.075,
                once: false,
            },
            jump: SpriteDef {
                image: image::Id::MbJump,
                first_frame: 0,
                frame_count: 4,
                sheet_frames: 4,
                frame_time: 0.1,
                once: false,
            },
            hurt: SpriteDef {
                image: image::Id::MbHurt,
                first_frame: 0,
                frame_count: 9,
                sheet_frames: 9,
                frame_time: 0.075,
                once: true,
            },
        };
        self.estore.add(player_defs, &[]);

        let spawn_x = 48.0;
        let spawn_y = colliders.iter().map(|r| r.y).fold(f32::INFINITY, f32::min) - 32.0;

        self.estore.add(
            Player {
                pos: vec2(spawn_x, spawn_y),
                vel: Vec2::ZERO,
                facing: 1.0,
                grounded: false,
                anim: crate::scene::scene_one::components::Anim::Idle,
                hurt: false,
            },
            &children![
                Physics {
                    move_speed: 120.0,
                    gravity: 900.0,
                    jump_velocity: -320.0,
                    sprite_size: 32.0,
                    hitbox_w: 20.0,
                    hitbox_h: 30.0,
                },
                AnimFrame {
                    frame: 0,
                    elapsed: 0.0,
                    idle_delay: 0.0,
                    done: false,
                },
                Blink {
                    active: false,
                    timer: 0.0,
                    blinks_left: 0,
                    visible: true,
                    blink_count: 5,
                    blink_interval: 0.15,
                },
            ],
        );

        let factory = EnemyFactory::new();
        let enemy_defs = factory.sprite_defs();
        self.estore.add(enemy_defs, &[]);

        let mut patrol_points = map
            .as_ref()
            .map(|m| m.patrol_points.clone())
            .unwrap_or_default();
        patrol_points.sort_by(|a, b| a.x.total_cmp(&b.x));

        if patrol_points.len() >= 2 {
            let p1 = patrol_points[0];
            let p2 = patrol_points[1];
            let (enemy, children) = factory.build_enemy_one([p1, p2]);
            self.estore.add(enemy, &children);
        }

        self.sys.add_update(PlayerSys::new(
            self.estore,
            self.cfg,
            colliders.clone(),
            map_size_px,
        ));
        self.sys.add_update(EnemySys::new(self.estore, colliders));
        self.sys.add_update(HurtSys::new(self.estore));
        self.sys.add_update(AnimationSys::new(self.estore));
        self.sys.add_update(EnemyAnimationSys::new(self.estore));
        self.sys
            .add_draw(DrawSys::new(self.estore, self.cfg, map, self.assets));

        Box::pin(std::future::ready(()))
    }

    fn update(&mut self, ctx: &mut Context) {
        self.sys.update(ctx);
    }

    fn draw(&self, ctx: &Context) {
        self.sys.draw(ctx);
    }

    fn dispose(&mut self) {
        self.sys.clear();
    }
}
