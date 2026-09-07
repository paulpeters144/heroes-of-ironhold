mod components;
mod frames;
mod movement;

pub use components::{EnemyStats, RamHead};
pub use frames::RAM_HEAD_FRAME_COUNT;

pub(crate) use frames::{ATTACK_FRAMES, IDLE_FRAME, WALK_FRAMES};
pub(crate) use movement::{ATTACK_RANGE, FRAME_DURATION, MOVE_SPEED, MOVE_SPEED_VERTICAL};
