mod attack;
mod components;
mod effect;
mod frames;
mod movement;

pub use attack::AttackKind;
pub use components::{Knight, Shield, Sword};
pub use effect::{Effect, EffectKind};
pub use frames::{FrameOffsets, KNIGHT_FRAME_COUNT};

pub(crate) use attack::AttackPhase;
pub(crate) use effect::{
    GLOW_IN_FRACTION, HOLD_START, SLASH_LIFETIME, SLASH_OFFSET_X, SLASH_OFFSET_Y, THRUST_LIFETIME,
    THRUST_OFFSET_X,
};
pub(crate) use frames::{IDLE_FRAME, SWIPE_FRAME, THRUST_FRAME, WALK_FRAMES};
pub(crate) use movement::{MOVE_SPEED, MOVE_SPEED_VERTICAL, WALK_FRAME_DURATION};
