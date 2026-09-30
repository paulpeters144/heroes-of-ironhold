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

pub const COMBO_WINDOW: f32 = 0.8;

impl AttackKind {
    pub(crate) fn frame(self) -> usize {
        match self {
            AttackKind::Thrust => THRUST_FRAME,
            AttackKind::Swipe => SWIPE_FRAME,
        }
    }

    pub(crate) fn windup(self) -> f32 {
        match self {
            AttackKind::Thrust => 0.04,
            AttackKind::Swipe => 0.06,
        }
    }

    pub(crate) fn strike(self) -> f32 {
        match self {
            AttackKind::Thrust => 0.12,
            AttackKind::Swipe => 0.18,
        }
    }

    pub(crate) fn recovery(self) -> f32 {
        match self {
            AttackKind::Thrust => 0.04,
            AttackKind::Swipe => 0.08,
        }
    }
}

pub(crate) fn combo_attack(combo_count: usize) -> AttackKind {
    match combo_count % 5 {
        0 => AttackKind::Thrust,
        1 => AttackKind::Swipe,
        2 => AttackKind::Thrust,
        3 => AttackKind::Swipe,
        4 => AttackKind::Thrust,
        _ => AttackKind::Thrust,
    }
}
