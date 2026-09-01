/// Placeholder icon identities for skill slots. The draw system maps each
/// kind to a simple procedural glyph; swap for real textures later.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkillIconKind {
    Sword,
    Shield,
    Potion,
    Fireball,
    Crossed,
}

/// Parent marker for the skills bar. Children are `Skill` slots in bar order.
#[derive(Clone, Debug)]
pub struct SkillsWidget;

/// One slot in the skills bar; child of `SkillsWidget`. A slot with no
/// `SkillIcon` child renders as an empty slot. `key` is the letter that
/// activates this skill (`None` for empty slots); each skill carries a
/// distinct letter so the same key never maps to two skills.
#[derive(Clone, Copy, Debug)]
pub struct Skill {
    pub selected: bool,
    pub key: Option<char>,
}

/// Icon descriptor; child of `Skill`.
#[derive(Clone, Copy, Debug)]
pub struct SkillIcon {
    pub kind: SkillIconKind,
}
