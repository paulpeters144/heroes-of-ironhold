use crate::entity::{EnemyStats, HealthBar, PeonStats, RamHead};
use crate::prelude::*;
use macroquad::prelude::*;
use std::rc::Rc;

pub struct HealthBarSystem {
    store: Rc<EStore>,
}

impl HealthBarSystem {
    pub fn new(store: Rc<EStore>) -> Self {
        Self { store }
    }
}

impl System for HealthBarSystem {
    fn update(&mut self, _ctx: &mut Context) {
        let mut writes: Vec<(pico_entity_store::entity_ref::EntityRef, Vec2, f32, f32)> =
            Vec::new();

        for bar in self.store.all::<HealthBar>() {
            let Some(owner) = self.store.parent(&bar) else {
                continue;
            };

            let parent_info = self
                .store
                .get_by_id::<RamHead>(owner.id())
                .and_then(|e| {
                    let anim = self.store.get_child::<Animation>(&e);
                    let stats = self.store.get_child::<EnemyStats>(&e);
                    match (anim, stats) {
                        (Some(anim), Some(stats)) => {
                            Some((anim, stats.hp as f32 / stats.max_hp.max(1) as f32))
                        }
                        (Some(anim), None) => Some((anim, 1.0)),
                        _ => None,
                    }
                })
                .map(|(anim, percent)| {
                    let r = anim.rect();
                    (
                        bar.width,
                        bar.height,
                        r,
                        anim.z_idx,
                        percent.clamp(0.0, 1.0),
                    )
                })
                .or_else(|| {
                    self.store
                        .get_by_id::<crate::entity::Peon>(owner.id())
                        .and_then(|e| {
                            let anim = self.store.get_child::<Animation>(&e);
                            let stats = self.store.get_child::<PeonStats>(&e);
                            match (anim, stats) {
                                (Some(anim), Some(stats)) => {
                                    Some((anim, stats.hp as f32 / stats.max_hp.max(1) as f32))
                                }
                                (Some(anim), None) => Some((anim, 1.0)),
                                _ => None,
                            }
                        })
                        .map(|(anim, percent)| {
                            let r = anim.rect();
                            (
                                bar.width,
                                bar.height,
                                r,
                                anim.z_idx,
                                percent.clamp(0.0, 1.0),
                            )
                        })
                });

            let Some((width, height, parent_rect, parent_z, percent)) = parent_info else {
                continue;
            };

            let x = parent_rect.x + (parent_rect.w - width) * 0.5;
            let y = parent_rect.y - height;
            writes.push((bar.entity_ref(), vec2(x, y), parent_z + 0.5, percent));
        }

        for (eref, position, z_idx, percent) in writes {
            self.store.update::<HealthBar, _>(&eref, |bar| {
                bar.position = position;
                bar.z_idx = z_idx;
                bar.percent = percent;
            });
        }
    }
}
