use crate::entity::{AttackRect, EnemyFactory, EnemyStats, HealthBar, ImpactFrame, RamHead, RamHeadCfg};
use crate::prelude::*;
use crate::{images, Assets};
use macroquad::prelude::*;
use pico_entity_store::store::IntoChild;
use std::rc::Rc;

const SPAWN_INTERVAL: f32 = 2.0;
const SPAWN_Y: f32 = 200.0;
const SPAWN_OFFSET_X: f32 = 32.0;

pub struct RamHeadSpawnerSystem {
    store: Rc<EStore>,
    body: Texture2D,
    impact: Texture2D,
    spawn_x: f32,
    spawn_timer: f32,
}

impl RamHeadSpawnerSystem {
    pub fn new(store: Rc<EStore>, assets: &Assets, map_w: f32) -> Self {
        let body = assets.texture(images::Enemy::RamHead);
        let impact = assets.texture(images::Enemy::RamHeadHit);
        Self {
            store,
            body,
            impact,
            spawn_x: map_w + SPAWN_OFFSET_X,
            spawn_timer: SPAWN_INTERVAL,
        }
    }

    fn spawn_ram_head(&self) {
        let mut parts = EnemyFactory::create_ram_head(RamHeadCfg {
            body: self.body.clone(),
            impact: self.impact.clone(),
        });
        parts.body.position = vec2(self.spawn_x, SPAWN_Y);
        self.store.add(
            parts.marker,
            &[
                parts.body.into_child(),
                parts.collision_circle.into_child(),
                HealthBar::default().into_child(),
                EnemyStats::default().into_child(),
                parts.impact_frame.into_child(),
                AttackRect {
                    rects: Vec::new(),
                    visible: false,
                }
                .into_child(),
            ],
        );

        let enemy_id = self
            .store
            .all::<RamHead>()
            .map(|e| e.entity_ref().id())
            .last()
            .expect("ram head just added");

        if let Some(impact_frame) = self
            .store
            .get_by_id::<RamHead>(enemy_id)
            .and_then(|e| self.store.get_child::<ImpactFrame>(&e))
        {
            self.store
                .add(impact_frame, &[parts.impact_image.into_child()]);
        }
    }
}

impl System for RamHeadSpawnerSystem {
    fn update(&mut self, ctx: &mut Context) {
        self.spawn_timer -= ctx.dt;
        if self.spawn_timer <= 0.0 {
            self.spawn_timer = SPAWN_INTERVAL;
            self.spawn_ram_head();
        }
    }
}