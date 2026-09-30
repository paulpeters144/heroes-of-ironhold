use crate::entity::{
    AttackRect, EnemyFactory, EnemyStats, HealthBar, ImpactFrame, RamHead, RamHeadCfg,
    RamHeadSpawnZone,
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

pub struct RamHeadSpawnerSystem {
    store: Rc<EStore>,
    body: Texture2D,
    impact: Texture2D,
    spawn_queue: Rc<RefCell<VecDeque<SpawnRamHeadEvent>>>,
    _subs: Rc<SubCollection>,
}

impl RamHeadSpawnerSystem {
    pub fn new(store: Rc<EStore>, bus: Rc<EventBus>, assets: &Assets) -> Self {
        let body = assets.texture(images::Enemy::RamHead);
        let impact = assets.texture(images::Enemy::RamHeadHit);
        let spawn_queue = Rc::new(RefCell::new(VecDeque::new()));
        let subs = Rc::new(SubCollection::new());

        let queue_for_handler = spawn_queue.clone();
        subs.on::<SpawnRamHeadEvent>(&bus, move |event: &SpawnRamHeadEvent| {
            queue_for_handler.borrow_mut().push_back(event.clone());
        });

        Self {
            store,
            body,
            impact,
            spawn_queue,
            _subs: subs,
        }
    }

    fn spawn_ram_head(&self) {
        let Some(rect) = self.store.first::<RamHeadSpawnZone>().map(|z| z.rect) else {
            return;
        };
        let mut parts = EnemyFactory::create_ram_head(RamHeadCfg {
            body: self.body.clone(),
            impact: self.impact.clone(),
        });
        let x = rect.x + gen_range(0.0, rect.w);
        let y = rect.y + gen_range(0.0, rect.h);
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