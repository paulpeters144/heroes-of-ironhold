use super::skills::{Skill, SkillDirection, SkillIcon, SkillIconKind, SkillsWidget};

/// Factory config for one directional attack (icon + direction). `icon: None`
/// yields an empty square; `direction: None` is an untagged attack.
#[derive(Clone, Copy, Debug)]
pub struct SkillSlotCfg {
    pub icon: Option<SkillIconKind>,
    pub direction: Option<SkillDirection>,
}

impl SkillSlotCfg {
    pub fn icon(kind: SkillIconKind) -> Self {
        Self {
            icon: Some(kind),
            direction: None,
        }
    }

    pub fn empty() -> Self {
        Self {
            icon: None,
            direction: None,
        }
    }

    pub fn direction(mut self, direction: SkillDirection) -> Self {
        self.direction = Some(direction);
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
        let mut parts = Vec::with_capacity(slots.len());
        for slot in slots {
            let skill = Skill {
                direction: slot.direction,
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
