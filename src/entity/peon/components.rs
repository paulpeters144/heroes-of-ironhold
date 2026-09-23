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

impl PeonStats {
    /// Damage taken from a raw hit after armor mitigation, flooring at 1 so a
    /// successful hit always deals at least 1. When `in_consecration` is true
    /// the peon's armor is doubled first, so it deducts more from the incoming
    /// attack.
    pub fn receive_damage(&self, raw: i32, in_consecration: bool) -> i32 {
        let armor = if in_consecration {
            (self.armor as f32 * 2.0) as i32
        } else {
            self.armor
        };
        (raw - armor).max(1)
    }
}

impl Default for PeonStats {
    fn default() -> Self {
        PeonStats {
            hp: 80,
            max_hp: 80,
            strength: 10,
            armor: 4,
        }
    }
}
