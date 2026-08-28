use macroquad::prelude::{vec2, Vec2};

pub const KNIGHT_FRAME_COUNT: usize = 6;

pub(crate) const IDLE_FRAME: usize = 0;
pub(crate) const THRUST_FRAME: usize = 1;
pub(crate) const SWIPE_FRAME: usize = 2;
pub(crate) const WALK_FRAMES: [usize; 3] = [3, 4, 5];

#[derive(Clone, Debug)]
pub struct FrameOffsets {
    pub shield: Vec2,
    pub sword: Vec2,
    pub sword_frame: usize,
    pub shield_visible: bool,
    pub sword_visible: bool,
}

pub fn knight_offsets() -> [FrameOffsets; KNIGHT_FRAME_COUNT] {
    [
        FrameOffsets {
            shield: vec2(33.0, 16.0),
            sword: vec2(18.0, 12.0),
            sword_frame: 0,
            shield_visible: true,
            sword_visible: true,
        },
        FrameOffsets {
            shield: vec2(33.0, 16.0),
            sword: vec2(60.0, 8.0),
            sword_frame: 1,
            shield_visible: false,
            sword_visible: true,
        },
        FrameOffsets {
            shield: vec2(33.0, 16.0),
            sword: vec2(18.0, 12.0),
            sword_frame: 0,
            shield_visible: false,
            sword_visible: false,
        },
        FrameOffsets {
            shield: vec2(33.0, 16.0),
            sword: vec2(18.0, 12.0),
            sword_frame: 0,
            shield_visible: true,
            sword_visible: true,
        },
        FrameOffsets {
            shield: vec2(33.0, 16.0),
            sword: vec2(18.0, 12.0),
            sword_frame: 0,
            shield_visible: true,
            sword_visible: true,
        },
        FrameOffsets {
            shield: vec2(33.0, 16.0),
            sword: vec2(18.0, 12.0),
            sword_frame: 0,
            shield_visible: true,
            sword_visible: true,
        },
    ]
}
