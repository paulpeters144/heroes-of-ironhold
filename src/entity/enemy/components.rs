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
