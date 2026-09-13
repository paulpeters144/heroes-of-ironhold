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

