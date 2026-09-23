use crate::entity::{
    AreaRect, AttackRect, EnemyFactory, EnemyStats, HealthBar, ImpactFrame, RamHead, RamHeadCfg,
};
use crate::events::SpawnRamHeadEvent;
use crate::prelude::*;
use crate::{images, Assets};
use macroquad::prelude::*;
use macroquad::rand::gen_range;
use pico_entity_store::store::IntoChild;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

const SPAWN_AREA_Y: f32 = 125.0;
const SPAWN_AREA_W: f32 = 50.0;
const SPAWN_AREA_H: f32 = 200.0;

pub struct RamHeadSpawnerSystem {
    store: Rc<EStore>,
    body: Texture2D,
    impact: Texture2D,
    spawn_area_x: f32,
    spawn_queue: Rc<RefCell<VecDeque<SpawnRamHeadEvent>>>,
    _subs: Rc<SubCollection>,
}

impl RamHeadSpawnerSystem {
    pub fn new(store: Rc<EStore>, bus: Rc<EventBus>, assets: &Assets, map_w: f32) -> Self {
        let body = assets.texture(images::Enemy::RamHead);
        let impact = assets.texture(images::Enemy::RamHeadHit);
        let spawn_queue = Rc::new(RefCell::new(VecDeque::new()));
        let subs = Rc::new(SubCollection::new());

        let queue_for_handler = spawn_queue.clone();
        subs.on::<SpawnRamHeadEvent>(&bus, move |event: &SpawnRamHeadEvent| {
            queue_for_handler.borrow_mut().push_back(event.clone());
        });

        let spawn_area_x = map_w - SPAWN_AREA_W;
        store.add(
            AreaRect {
                rect: Rect::new(spawn_area_x, SPAWN_AREA_Y, SPAWN_AREA_W, SPAWN_AREA_H),
            },
            &[],
        );

        Self {
            store,
            body,
            impact,
            spawn_area_x,
            spawn_queue,
            _subs: subs,
        }
    }

    fn spawn_ram_head(&self) {
        let mut parts = EnemyFactory::create_ram_head(RamHeadCfg {
            body: self.body.clone(),
            impact: self.impact.clone(),
        });
        let x = self.spawn_area_x + gen_range(0.0, SPAWN_AREA_W);
        let y = SPAWN_AREA_Y + gen_range(0.0, SPAWN_AREA_H);
        parts.body.position = vec2(x, y);
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
    fn update(&mut self, _ctx: &mut Context) {
        while let Some(event) = self.spawn_queue.borrow_mut().pop_front() {
            for _ in 0..event.count {
                self.spawn_ram_head();
            }
        }
    }
}