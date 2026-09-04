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
}

impl Default for HeroStats {
    fn default() -> Self {
        HeroStats {
            name: "Knight",
            hp: 145,
            max_hp: 150,
            mp: 80,
            max_mp: 100,
            xp: 2500,
            max_xp: 5000,
            level: 5,
        }
    }
}
