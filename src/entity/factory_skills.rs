use crate::entity::skills::{Skill, SkillIcon, SkillIconKind, SkillsWidget};
use crate::EStore;
use pico_entity_store::store::{ChildSource, IntoChild};

/// One slot to spawn in the skills bar. `icon: None` yields an empty slot.
#[derive(Clone, Copy, Debug)]
pub struct SkillSlotCfg {
    pub icon: Option<SkillIconKind>,
    pub selected: bool,
}

impl SkillSlotCfg {
    pub fn icon(kind: SkillIconKind) -> Self {
        Self {
            icon: Some(kind),
            selected: false,
        }
    }

    pub fn empty() -> Self {
        Self {
            icon: None,
            selected: false,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

pub struct SkillsFactory;

impl SkillsFactory {
    /// Spawns the `SkillsWidget -> Skill -> SkillIcon` hierarchy. Slot order
    /// in the bar matches `slots` order; any count works.
    pub fn spawn(store: &EStore, slots: &[SkillSlotCfg]) {
        store.add(SkillsWidget, &[]);
        let mut next_key = 'a' as u32;
        for slot in slots {
            let key = if slot.icon.is_some() {
                let k = char::from_u32(next_key).expect("key in range");
                next_key += 1;
                Some(k)
            } else {
                None
            };
            let skill = Skill {
                selected: slot.selected,
                key,
            };
            match slot.icon {
                Some(kind) => store.add(skill, &[SkillIcon { kind }.into_child()]),
                None => store.add(skill, &[]),
            }
            Self::attach_last_skill(store);
        }
    }

    /// `EStore::add` discards the new entity's ref, so re-fetch the skill we
    /// just pushed (storage is append-ordered) and link it under the widget.
    fn attach_last_skill(store: &EStore) {
        let skill_ref = store
            .all::<Skill>()
            .map(|s| s.entity_ref())
            .last()
            .expect("skill just added");
        let widget = store.first::<SkillsWidget>().expect("skills widget");
        store.add(widget, &[ChildSource::Existing(skill_ref)]);
    }
}
