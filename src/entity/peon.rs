mod components;
mod frames;

pub use components::{Peon, PeonStats};
pub use frames::PEON_FRAME_COUNT;

pub(crate) use frames::{PEON_ATTACK_HIT1_FRAMES, PEON_ATTACK_HIT2_FRAMES, PEON_WALK_FRAMES};
