mod attack;
mod components;
mod effect;
mod frames;
mod movement;

pub use attack::AttackKind;
pub use components::{
    Consecration, DivineStance, Facing, GuardianShield, Knight, KnightLock, Shield, Sword,
};
pub use effect::{Effect, EffectKind};
pub use frames::FrameOffsets;

pub(crate) use attack::AttackPhase;
pub(crate) use effect::{GLOW_IN_FRACTION, HOLD_START, SLASH_LIFETIME, THRUST_LIFETIME};
pub(crate) use frames::{IDLE_FRAME, SWIPE_FRAME, THRUST_FRAME, WALK_FRAMES};
pub(crate) use movement::{
    DOUBLE_TAP_WINDOW, MOVE_SPEED, MOVE_SPEED_VERTICAL, REVERSE_MULT, WALK_FRAME_DURATION,
};
