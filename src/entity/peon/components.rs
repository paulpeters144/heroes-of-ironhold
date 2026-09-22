/// A hero peon — a lane-minion-style NPC that marches forward and attacks.
#[derive(Clone, Debug)]
pub struct Peon;

#[derive(Clone, Copy, Debug)]
pub struct PeonStats {
    pub hp: i32,
    pub max_hp: i32,
    pub strength: i32,
    pub armor: i32,
}

impl Default for PeonStats {
    fn default() -> Self {
        PeonStats {
            hp: 80,
            max_hp: 80,
            strength: 10,
            armor: 0,
        }
    }
}
