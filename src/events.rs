use crate::entity::{SkillDirection, SkillIconKind};
use macroquad::prelude::Rect;

#[derive(Clone, Debug)]
pub struct AttackEvent {
    pub attacker: u64,
    pub targets: Vec<u64>,
}

#[derive(Clone, Debug)]
pub struct HealthChangeEvent {
    pub entity: u64,
    pub amount: i32,
    pub rect: Rect,
}

#[derive(Clone, Debug)]
pub struct HitEvent {
    pub victim: u64,
    pub attacker: u64,
}

#[derive(Clone, Debug)]
pub struct EnemyAttackEvent {
    pub target: u64,
    pub damage: i32,
}

#[derive(Clone, Debug)]
pub struct EnemyDeathEvent {
    pub enemy: u64,
}

#[derive(Clone, Debug)]
pub struct SkillCastEvent {
    pub caster: u64,
    pub direction: SkillDirection,
    pub kind: SkillIconKind,
}
