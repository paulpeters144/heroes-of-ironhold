use crate::entity::knight::{Shield, Sword};
use crate::systems::Update;
use crate::{Animation, Context, EStore, StaticImage};
use pico_entity_store::entity_ref::EntityRef;
use std::collections::HashMap;
use std::rc::Rc;

const SUB_PART_STEP: f32 = 0.01;

#[derive(Clone, Copy)]
enum Kind {
    Anim,
    Static,
}

struct Entry {
    eref: EntityRef,
    kind: Kind,
    bottom: f32,
    order: f32,
}

pub struct ZSortSystem {
    store: Rc<EStore>,
}

impl ZSortSystem {
    pub fn new(store: Rc<EStore>) -> Self {
        Self { store }
    }

    fn owner_of(&self, eref: &EntityRef, kind: Kind) -> u64 {
        let parent = match kind {
            Kind::Anim => self
                .store
                .get_by_id::<Animation>(eref.id())
                .and_then(|r| self.store.parent(&r)),
            Kind::Static => self
                .store
                .get_by_id::<StaticImage>(eref.id())
                .and_then(|r| self.store.parent(&r)),
        };

        let Some(parent) = parent else {
            return eref.id();
        };

        let is_part = self.store.get_by_id::<Sword>(parent.id()).is_some()
            || self.store.get_by_id::<Shield>(parent.id()).is_some();

        if is_part {
            let grandparent = self
                .store
                .get_by_id::<Sword>(parent.id())
                .and_then(|r| self.store.parent(&r))
                .or_else(|| {
                    self.store
                        .get_by_id::<Shield>(parent.id())
                        .and_then(|r| self.store.parent(&r))
                });
            grandparent.map(|r| r.id()).unwrap_or(parent.id())
        } else {
            parent.id()
        }
    }
}

impl Update for ZSortSystem {
    fn update(&mut self, _ctx: &mut Context) {
        let mut entries: Vec<Entry> = Vec::new();

        for animation in self.store.all::<Animation>() {
            let rect = animation.rect();
            entries.push(Entry {
                eref: animation.entity_ref(),
                kind: Kind::Anim,
                bottom: rect.y + rect.h,
                order: animation.z_idx,
            });
        }

        for image in self.store.all::<StaticImage>() {
            let rect = image.rect();
            entries.push(Entry {
                eref: image.entity_ref(),
                kind: Kind::Static,
                bottom: rect.y + rect.h,
                order: image.z_idx,
            });
        }

        if entries.is_empty() {
            return;
        }

        let mut owners: HashMap<u64, (f32, Vec<usize>)> = HashMap::new();
        for (idx, entry) in entries.iter().enumerate() {
            let owner = self.owner_of(&entry.eref, entry.kind);
            let group = owners.entry(owner).or_insert((f32::MIN, Vec::new()));
            if entry.bottom > group.0 {
                group.0 = entry.bottom;
            }
            group.1.push(idx);
        }

        let mut sorted: Vec<(u64, f32, Vec<usize>)> = owners
            .into_iter()
            .map(|(id, (feet, idxs))| (id, feet, idxs))
            .collect();
        sorted.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        for (rank, (_, _, idxs)) in sorted.into_iter().enumerate() {
            let mut idxs = idxs;
            idxs.sort_by(|&a, &b| {
                entries[a]
                    .order
                    .partial_cmp(&entries[b].order)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            for (part_idx, &i) in idxs.iter().enumerate() {
                let z = rank as f32 + part_idx as f32 * SUB_PART_STEP;
                let entry = &entries[i];
                match entry.kind {
                    Kind::Anim => {
                        self.store
                            .update::<Animation, _>(&entry.eref, |a| a.z_idx = z);
                    }
                    Kind::Static => {
                        self.store
                            .update::<StaticImage, _>(&entry.eref, |s| s.z_idx = z);
                    }
                }
            }
        }
    }
}
