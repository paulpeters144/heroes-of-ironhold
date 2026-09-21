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

/// Fired by a skill system the frame it actually starts a cooldown (a cast
/// that was rejected because the skill was still cooling down does not fire).
/// `duration` is the full cooldown length so the UI can draw progress without
/// knowing each skill's tuning.
#[derive(Clone, Debug)]
pub struct SkillCooldownEvent {
    pub kind: SkillIconKind,
    pub duration: f32,
}

/// Fired by a zone skill (Consecration, Divine Stance) the frame its area is
/// summoned. `duration` is the zone's full lifetime, which is also how long
/// the skill cannot be recast. Lets the UI show a distinct "zone active" state
/// rather than painting a persistent area as a long cooldown.
#[derive(Clone, Debug)]
pub struct SkillActiveEvent {
    pub kind: SkillIconKind,
    pub duration: f32,
}

/// Fired the frame a zone skill's area despawns, so the UI can drop its
/// "zone active" indicator and flash the slot as ready again.
#[derive(Clone, Debug)]
pub struct SkillActiveEndEvent {
    pub kind: SkillIconKind,
}
