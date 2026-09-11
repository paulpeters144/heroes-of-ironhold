use crate::entity::skills::{Skill, SkillIcon, SkillIconKind, SkillsWidget};

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

#[derive(Clone, Debug)]
pub struct SkillSlotParts {
    pub skill: Skill,
    pub icon: Option<SkillIcon>,
}

#[derive(Clone, Debug)]
pub struct SkillsParts {
    pub widget: SkillsWidget,
    pub slots: Vec<SkillSlotParts>,
}

pub struct SkillsFactory;

impl SkillsFactory {
    pub fn create(slots: &[SkillSlotCfg]) -> SkillsParts {
        let mut next_key = 'a' as u32;
        let mut parts = Vec::with_capacity(slots.len());
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
            let icon = slot.icon.map(|kind| SkillIcon { kind });
            parts.push(SkillSlotParts { skill, icon });
        }
        SkillsParts {
            widget: SkillsWidget,
            slots: parts,
        }
    }
}
