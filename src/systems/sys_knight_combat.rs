use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use crate::entity::enemy::{EnemyStats, RamHead};
use crate::entity::hero::HeroStats;
use crate::entity::knight::Knight;
use crate::events::{AttackEvent, EnemyAttackEvent, EnemyDeathEvent, HealthChangeEvent, HitEvent};
use crate::systems::Update;
use crate::{Animation, Context, EStore, EventBus, SubCollection};

pub struct KnightCombatSystem {
    store: Rc<EStore>,
    bus: Rc<EventBus>,
    attack_queue: Rc<RefCell<VecDeque<AttackEvent>>>,
    enemy_attack_queue: Rc<RefCell<VecDeque<EnemyAttackEvent>>>,
    _subs: Rc<SubCollection>,
}

impl KnightCombatSystem {
    pub fn new(store: Rc<EStore>, bus: Rc<EventBus>) -> Self {
        let attack_queue = Rc::new(RefCell::new(VecDeque::new()));
        let enemy_attack_queue = Rc::new(RefCell::new(VecDeque::new()));
        let subs = Rc::new(SubCollection::new());

        let attack_queue_for_handler = attack_queue.clone();
        subs.on::<AttackEvent>(&bus, move |event: &AttackEvent| {
            attack_queue_for_handler.borrow_mut().push_back(event.clone());
        });

        let enemy_attack_queue_for_handler = enemy_attack_queue.clone();
        subs.on::<EnemyAttackEvent>(&bus, move |event: &EnemyAttackEvent| {
            enemy_attack_queue_for_handler.borrow_mut().push_back(event.clone());
        });

        Self {
            store,
            bus,
            attack_queue,
            enemy_attack_queue,
            _subs: subs,
        }
    }
}

impl Update for KnightCombatSystem {
    fn update(&mut self, _ctx: &mut Context) {
        while let Some(event) = self.attack_queue.borrow_mut().pop_front() {
            let damage = self
                .store
                .get_by_id::<Knight>(event.attacker)
                .and_then(|knight| self.store.get_child::<HeroStats>(&knight))
                .map(|stats| stats.strength)
                .unwrap_or(1)
                .max(1);

            for target in event.targets {
                let stats_ref = self
                    .store
                    .get_by_id::<RamHead>(target)
                    .and_then(|enemy| self.store.get_child::<EnemyStats>(&enemy))
                    .map(|stats| stats.entity_ref());
                if let Some(stats_ref) = stats_ref {
                    let was_alive = self
                        .store
                        .get_by_id::<EnemyStats>(stats_ref.id())
                        .map(|stats| stats.hp > 0)
                        .unwrap_or(false);

                    self.store.update::<EnemyStats, _>(&stats_ref, |stats| {
                        stats.hp = (stats.hp - damage).max(0);
                    });

                    let is_dead = self
                        .store
                        .get_by_id::<EnemyStats>(stats_ref.id())
                        .map(|stats| stats.hp <= 0)
                        .unwrap_or(false);

                    if was_alive && is_dead {
                        self.bus.fire(&EnemyDeathEvent { enemy: target });
                    }

                    let rect = self
                        .store
                        .get_by_id::<RamHead>(target)
                        .and_then(|enemy| self.store.get_child::<Animation>(&enemy))
                        .map(|anim| anim.rect());
                    if let Some(rect) = rect {
                        self.bus.fire(&HealthChangeEvent {
                            entity: target,
                            amount: -(damage),
                            rect,
                        });
                    }
                }

                self.bus.fire(&HitEvent {
                    victim: target,
                    attacker: event.attacker,
                });
            }
        }

        while let Some(event) = self.enemy_attack_queue.borrow_mut().pop_front() {
            let damage = self
                .store
                .get_by_id::<RamHead>(event.attacker)
                .and_then(|enemy| self.store.get_child::<EnemyStats>(&enemy))
                .map(|stats| stats.strength)
                .unwrap_or(1)
                .max(1);

            // End the guard chain in one statement so no read guard is
            // still alive when `update` takes the store write lock.
            let Some((stats_ref, rect)) = self
                .store
                .get_by_id::<Knight>(event.target)
                .and_then(|knight| {
                    let stats_ref = self.store.get_child::<HeroStats>(&knight)?.entity_ref();
                    let rect = self.store.get_child::<Animation>(&knight)?.rect();
                    Some((stats_ref, rect))
                })
            else {
                continue;
            };

            self.store.update::<HeroStats, _>(&stats_ref, |stats| {
                stats.hp = (stats.hp - damage).max(0);
            });
            self.bus.fire(&HealthChangeEvent {
                entity: event.target,
                amount: -damage,
                rect,
            });

            self.bus.fire(&HitEvent {
                victim: event.target,
                attacker: event.attacker,
            });
        }
    }
}
