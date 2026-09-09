use crate::entity::enemy::RamHead;
use crate::entity::knight::{AttackArea, Knight, Sword};
use crate::entity::player::PlayerOne;
use crate::systems::Update;
use crate::{Animation, AttackEvent, Context, EStore, EventBus};
use std::rc::Rc;

pub struct AttackHitSystem {
    store: Rc<EStore>,
    bus: Rc<EventBus>,
    prev_visible: bool,
}

impl AttackHitSystem {
    pub fn new(store: Rc<EStore>, bus: Rc<EventBus>) -> Self {
        Self {
            store,
            bus,
            prev_visible: false,
        }
    }
}

impl Update for AttackHitSystem {
    fn update(&mut self, _ctx: &mut Context) {
        let Some(knight) = self
            .store
            .first::<PlayerOne>()
            .and_then(|p| self.store.get_child::<Knight>(&p))
            .map(|k| k.entity_ref())
        else {
            return;
        };

        let Some(area) = self
            .store
            .first::<PlayerOne>()
            .and_then(|p| self.store.get_child::<Knight>(&p))
            .and_then(|k| self.store.get_child::<Sword>(&k))
            .and_then(|s| self.store.get_child::<AttackArea>(&s))
        else {
            return;
        };

        let mut targets = Vec::new();
        for enemy in self.store.all::<RamHead>() {
            if let Some(body) = self.store.get_child::<Animation>(&enemy) {
                let rect = body.rect();
                if area.rects.iter().any(|r| r.overlaps(&rect)) {
                    targets.push(enemy.entity_ref().id());
                }
            }
        }

        if area.visible && !self.prev_visible && !targets.is_empty() {
            self.bus.fire(&AttackEvent {
                attacker: knight.id(),
                targets,
            });
        }
        self.prev_visible = area.visible;
    }
}
