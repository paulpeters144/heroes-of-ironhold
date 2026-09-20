use crate::entity::knight::{Knight, KnightLock, Sword};
use crate::entity::{PlayerOne, RamHead};
use crate::events::AttackEvent;
use crate::prelude::*;
use crate::util::{did_attack, image_data_for};
use macroquad::prelude::{Image, Texture2D};
use std::rc::Rc;

pub struct KnightAttackHitSystem {
    store: Rc<EStore>,
    bus: Rc<EventBus>,
    prev_visible: bool,
    sheet_cache: Option<(Texture2D, Image)>,
}

impl KnightAttackHitSystem {
    pub fn new(store: Rc<EStore>, bus: Rc<EventBus>) -> Self {
        Self {
            store,
            bus,
            prev_visible: false,
            sheet_cache: None,
        }
    }
}

impl System for KnightAttackHitSystem {
    fn update(&mut self, _ctx: &mut Context) {
        // Skip the knight while a skill holds it (thrust pose is cosmetic).
        if self
            .store
            .first::<PlayerOne>()
            .and_then(|p| self.store.get_child::<Knight>(&p))
            .and_then(|k| self.store.get_child::<KnightLock>(&k))
            .is_some()
        {
            return;
        }

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
            .and_then(|s| self.store.get_child::<AttackRect>(&s))
        else {
            return;
        };

        let mut targets = Vec::new();
        for enemy in self.store.all::<RamHead>() {
            if let Some(body) = self.store.get_child::<Animation>(&enemy) {
                let data = image_data_for(&body, &mut self.sheet_cache);
                if area.rects.iter().any(|r| did_attack(*r, &data)) {
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
