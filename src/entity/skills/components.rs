/// Placeholder icon identities for skill slots. The draw system maps each
/// kind to a simple procedural glyph; swap for real textures later.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkillIconKind {
    Sword,
    Shield,
    Potion,
    Fireball,
    Crossed,
    ShieldCycle,
    DivineArea,
}

/// Arrow direction this attack answers to while the skill selector is held.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkillDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Parent marker for the skills bar. Children are the four direction-tagged
/// `Skill` attacks and a `LastUsedSkill` marker.
#[derive(Clone, Debug)]
pub struct SkillsWidget;

/// One of the four directional attacks in the skill selector; child of
/// `SkillsWidget`. `direction` is the arrow that selects it. An attack with no
/// `SkillIcon` child renders as an empty square.
#[derive(Clone, Copy, Debug)]
pub struct Skill {
    pub direction: Option<SkillDirection>,
}

/// Icon descriptor; child of `Skill`.
#[derive(Clone, Copy, Debug)]
pub struct SkillIcon {
    pub kind: SkillIconKind,
}

/// Most recently fired attack, rendered in the bar's attack slot and used as
/// the initial selection when the selector opens. `icon: None` / `direction:
/// None` until an attack fires.
#[derive(Clone, Copy, Debug, Default)]
pub struct LastUsedSkill {
    pub icon: Option<SkillIconKind>,
    pub direction: Option<SkillDirection>,
}
