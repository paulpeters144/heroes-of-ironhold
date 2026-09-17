#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Facing {
    Left,
    Right,
}

#[derive(Clone, Debug)]
pub struct Knight;

impl Knight {
    pub fn offsets() -> [super::frames::FrameOffsets; super::frames::KNIGHT_FRAME_COUNT] {
        super::frames::knight_offsets()
    }
}

#[derive(Clone, Debug)]
pub struct Shield;

#[derive(Clone, Debug)]
pub struct Sword;

/// A named lock holding the knight still while a skill (e.g. Swords) is
/// casting. While a `KnightLock` is a child of the knight, the control system
/// skips movement and the attack systems skip the knight.
#[derive(Clone, Debug)]
pub struct KnightLock {
    pub by: &'static str,
}

/// Marker for the knight's active guardian-shield ward. Present in the store
/// only while the shield is up; combat queries it to boost the knight's armor.
#[derive(Clone, Debug)]
pub struct GuardianShield;

/// Marker for the knight's active divine-area healing zone. Present in the
/// store only while the zone is on the ground; the `DivineAreaSystem` spawns
/// it on cast and removes it when the zone expires.
#[derive(Clone, Debug)]
pub struct DivineArea;
