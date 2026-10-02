use crate::entity::{ImpactBurst, ProceduralDrawable, ProceduralEffect};
use crate::events::RamHeadSplatEvent;
use crate::prelude::*;
use pico_entity_store::entity_ref::EntityRef;
use std::cell::RefCell;
use std::rc::Rc;

const BURST_DURATION: f32 = 0.18;
const SPLAT_Z_OFFSET: f32 = 5.005;

pub struct RamHeadSplatSystem {
    store: Rc<EStore>,
    queue: Rc<RefCell<Vec<RamHeadSplatEvent>>>,
    _subs: Rc<SubCollection>,
}

impl RamHeadSplatSystem {
    pub fn new(store: Rc<EStore>, bus: Rc<EventBus>) -> Self {
        let queue = Rc::new(RefCell::new(Vec::new()));
        let subs = Rc::new(SubCollection::new());
        let queue_for_handler = queue.clone();
        subs.on::<RamHeadSplatEvent>(&bus, move |event: &RamHeadSplatEvent| {
            queue_for_handler.borrow_mut().push(event.clone());
        });
        Self {
            store,
            queue,
            _subs: subs,
        }
    }

    fn hit_entity_z(&self, target: u64) -> f32 {
        self.store
            .get_by_id::<crate::entity::Knight>(target)
            .and_then(|k| self.store.get_child::<Animation>(&k))
            .map(|a| a.z_idx)
            .unwrap_or(0.0)
    }
}

impl System for RamHeadSplatSystem {
    fn update(&mut self, ctx: &mut Context) {
        let events: Vec<RamHeadSplatEvent> = self.queue.borrow_mut().drain(..).collect();

        for event in events {
            let hit_z = self.hit_entity_z(event.target);
            let burst = ImpactBurst::new(event.position, BURST_DURATION);
            let drawable = ProceduralDrawable {
                effect: ProceduralEffect::ImpactBurst(burst),
                z_idx: hit_z + SPLAT_Z_OFFSET,
                visible: true,
            };
            self.store.add(drawable, &[]);
        }

        let mut expired: Vec<EntityRef> = Vec::new();
        for mut burst in self.store.all_mut::<ProceduralDrawable>() {
            if let ProceduralEffect::ImpactBurst(ref mut data) = burst.effect {
                data.age += ctx.dt;
                if data.age >= data.duration {
                    expired.push(burst.entity_ref());
                }
            }
        }
        if !expired.is_empty() {
            self.store.remove(&expired);
        }
    }
}
