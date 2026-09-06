/// The base stats every hero has. Each variant maps to a name/description
/// pair used for tooltips and the character sheet; numeric values live on
/// `HeroStats`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoreStat {
    HealthPoints,
    MagicPoints,
    Strength,
    Armor,
    Intelligence,
    Resistances,
    Evasion,
    Accuracy,
    CriticalStrike,
}

impl CoreStat {
    pub const ALL: [CoreStat; 9] = [
        CoreStat::HealthPoints,
        CoreStat::MagicPoints,
        CoreStat::Strength,
        CoreStat::Armor,
        CoreStat::Intelligence,
        CoreStat::Resistances,
        CoreStat::Evasion,
        CoreStat::Accuracy,
        CoreStat::CriticalStrike,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            CoreStat::HealthPoints => "Health Points (HP)",
            CoreStat::MagicPoints => "Magic Points (MP)",
            CoreStat::Strength => "Strength (STR)",
            CoreStat::Armor => "Armor (ARM)",
            CoreStat::Intelligence => "Intelligence (INT)",
            CoreStat::Resistances => "Resistances (RES)",
            CoreStat::Evasion => "Evasion (EVA)",
            CoreStat::Accuracy => "Accuracy (ACC)",
            CoreStat::CriticalStrike => "Critical Strike (CTR)",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            CoreStat::HealthPoints => {
                "Determines the maximum amount of damage a hero can take before being defeated."
            }
            CoreStat::MagicPoints => {
                "A universal resource pool used to power active skills, spells, and special combat attacks."
            }
            CoreStat::Strength => {
                "Increases physical damage, knockback, stun duration, and reduces knockback taken."
            }
            CoreStat::Armor => {
                "Mitigates incoming physical damage from melee attacks and projectiles."
            }
            CoreStat::Intelligence => {
                "Increases magical power, spell effectiveness, and maximum Mana pool."
            }
            CoreStat::Resistances => {
                "Reduces the effect of magical damage."
            }
            CoreStat::Evasion => {
                "Increases the chances of dodging incoming physical attacks, both physical and magical."
            }
            CoreStat::Accuracy => {
                "Determines the likelihood of landing a successful strike."
            }
            CoreStat::CriticalStrike => {
                "The percentage chance for an attack or spell to deal multiplied damage."
            }
        }
    }
}

/// Base stats shared by every hero. Percentile stats (evasion, accuracy,
/// critical) are stored as integer percentages.
#[derive(Clone, Copy, Debug)]
pub struct HeroStats {
    pub name: &'static str,
    pub hp: i32,
    pub max_hp: i32,
    pub mp: i32,
    pub max_mp: i32,
    pub xp: i32,
    pub max_xp: i32,
    pub level: u32,
    pub strength: i32,
    pub armor: i32,
    pub intelligence: i32,
    pub resistances: i32,
    pub evasion: i32,
    pub accuracy: i32,
    pub critical: i32,
}

impl Default for HeroStats {
    fn default() -> Self {
        HeroStats {
            name: "Knight",
            hp: 150,
            max_hp: 150,
            mp: 100,
            max_mp: 100,
            xp: 2500,
            max_xp: 5000,
            level: 5,
            strength: 12,
            armor: 8,
            intelligence: 5,
            resistances: 5,
            evasion: 5,
            accuracy: 95,
            critical: 10,
        }
    }
}
