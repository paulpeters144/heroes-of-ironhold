use super::frames::{SWIPE_FRAME, THRUST_FRAME};

#[derive(Clone, Copy, PartialEq)]
pub enum AttackKind {
    Thrust,
    Swipe,
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum AttackPhase {
    Idle,
    Windup,
    Strike,
    Recovery,
}

impl AttackKind {
    pub(crate) fn frame(self) -> usize {
        match self {
            AttackKind::Thrust => THRUST_FRAME,
            AttackKind::Swipe => SWIPE_FRAME,
        }
    }

    pub(crate) fn windup(self) -> f32 {
        match self {
            AttackKind::Thrust => 0.05,
            AttackKind::Swipe => 0.05,
        }
    }

    pub(crate) fn strike(self) -> f32 {
        match self {
            AttackKind::Thrust => 0.15,
            AttackKind::Swipe => 0.15,
        }
    }

    pub(crate) fn recovery(self) -> f32 {
        0.05
    }
}
