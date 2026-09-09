#[derive(Clone, Debug)]
pub struct RamHead;

/// Flat combat stats for an enemy. Enemies don't level; these values are
/// final and read directly by combat systems.
#[derive(Clone, Copy, Debug)]
pub struct EnemyStats {
    pub name: &'static str,
    pub hp: i32,
    pub max_hp: i32,
    pub strength: i32,
    pub armor: i32,
    pub evasion: i32,
    pub accuracy: i32,
    pub critical: i32,
}

impl Default for EnemyStats {
    fn default() -> Self {
        EnemyStats {
            name: "Ram Head",
            hp: 60,
            max_hp: 60,
            strength: 8,
            armor: 0,
            evasion: 5,
            accuracy: 90,
            critical: 5,
        }
    }
}
