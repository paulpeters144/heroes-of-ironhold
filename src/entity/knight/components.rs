use macroquad::prelude::Rect;

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

/// A named lock holding the knight still while a skill (e.g. Blade Barrage) is
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

/// Marker for the knight's active divine-stance healing zone. Present in the
/// store only while the zone is on the ground; the `DivineStanceSystem` spawns
/// it on cast and removes it when the zone expires.
#[derive(Clone, Debug)]
pub struct DivineStance;

/// Marker for the knight's active consecration zone. Present in the store only
/// while the holy ground persists; combat queries its `rect` to boost the
/// knight's armor by 25% while the knight stands inside it.
#[derive(Clone, Debug)]
pub struct Consecration {
    pub rect: Rect,
}
