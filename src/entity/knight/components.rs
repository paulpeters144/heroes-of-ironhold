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
