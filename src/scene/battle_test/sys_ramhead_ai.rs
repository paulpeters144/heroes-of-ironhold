use crate::entity::enemy::{EnemyStats, RamHead, ATTACK_FRAMES, WALK_FRAMES};
use crate::entity::knight::Knight;
use crate::entity::{HeroStats, Peon, PeonStats, PlayerOne};
use crate::events::{EnemyAttackEvent, HitEvent};
use crate::prelude::*;
use macroquad::prelude::{vec2, Color, Rect, Vec2};
use pico_entity_store::prelude::EntityRef;
use std::collections::HashMap;
use std::rc::Rc;

// Stalk speed and walk-frame timing.
const MOVE_SPEED: f32 = 60.0;
const FRAME_DURATION: f32 = 0.14;
const WALK_ANIM_MIN_SPEED: f32 = 8.0;

// Melee attack: how far ahead the hit strip reaches and its timing.
const ATTACK_REACH: f32 = 26.0;
const ATTACK_DURATION: f32 = 0.30;
const ATTACK_FRAME_DURATION: f32 = 0.06;
const ATTACK_COOLDOWN: f32 = 0.55;

// Charge behavior: a ram head stalks until its prey is in range, winds up
// (telegraphed), then bursts forward in a locked direction the player can
// sidestep. Contact during the charge (or adjacency while stalking) triggers
// the melee attack.
const CHARGE_TRIGGER_RANGE: f32 = 180.0;
const WINDUP_SECS: f32 = 0.45;
const CHARGE_SPEED: f32 = 175.0;
const CHARGE_MAX_SECS: f32 = 0.8;
const CHARGE_COOLDOWN: f32 = 1.4;
const RECOVER_SECS: f32 = 0.35;

// Steering: velocity easing and boids-style separation between ram heads.
const ACCEL: f32 = 420.0;
const CHARGE_ACCEL: f32 = 1200.0;
const SEPARATION_RADIUS: f32 = 46.0;
const SEPARATION_WEIGHT: f32 = 90.0;

// Keep ram heads within the map's vertical bounds.
const Y_BOUND_MIN: f32 = 40.0;
const Y_BOUND_MAX: f32 = 360.0;

const NORMAL_TINT: Color = Color::new(1.0, 1.0, 1.0, 1.0);
const WINDUP_TINT: Color = Color::new(1.0, 0.55, 0.55, 1.0);

/// The ram head's movement mode. It never stands still by design: it stalks,
/// telegraphs, charges, and recovers.
#[derive(Clone, Copy, PartialEq, Eq)]
enum RamMode {
    Stalk,
    Windup,
    Charge,
    Recover,
}

/// Per-ram brain: steering velocity plus the charge mode machine and the
/// melee attack state.
#[derive(Clone)]
struct RamBrain {
    mode: RamMode,
    vel: Vec2,          // current smoothed velocity
    facing: Vec2,       // unit direction the ram faces this frame; defaults to left (-1, 0)
    charge_dir: Vec2,   // direction locked in when the charge launches
    mode_timer: f32,    // remaining time in Windup/Charge/Recover
    charge_cooldown: f32, // seconds until the next charge is allowed
    walk_step: usize,   // index into WALK_FRAMES
    frame_elapsed: f32, // walk/attack-frame animation accumulator
    attack_cooldown: f32, // seconds until the next attack is allowed
    attacking: bool,    // true while a melee attack is active (opens AttackRect)
    attack_timer: f32,  // remaining time of the active attack; plays ATTACK_FRAMES
    attack_step: usize, // index into ATTACK_FRAMES
    attack_target: Option<u64>, // entity id of the hero/peon being attacked
}

impl Default for RamBrain {
    fn default() -> Self {
        RamBrain {
            mode: RamMode::Stalk,
            vel: Vec2::ZERO,
            facing: vec2(-1.0, 0.0),
            charge_dir: vec2(-1.0, 0.0),
            mode_timer: 0.0,
            charge_cooldown: 0.0,
            walk_step: 0,
            frame_elapsed: 0.0,
            attack_cooldown: 0.0,
            attacking: false,
            attack_timer: 0.0,
            attack_step: 0,
            attack_target: None,
        }
    }
}

struct AnimWrite {
    anim_ref: EntityRef,
    position: Vec2,
    frame: usize,
    flip_x: bool,
    tint: Color,
}

struct AttackRectWrite {
    area_ref: EntityRef,
    rects: Vec<Rect>,
    visible: bool,
}

pub struct RamHeadAiSystem {
    store: Rc<EStore>,
    bus: Rc<EventBus>,
    states: HashMap<u64, RamBrain>,
}

impl RamHeadAiSystem {
    /// The melee attack rectangle: an ATTACK_REACH-deep strip just in front of
    /// `body` toward `dir`, so the attack sits ahead of the ram and doesn't
    /// cover its own body.
    fn forward_rect(body: Rect, dir: Vec2) -> Rect {
        let mut r = body;
        if dir.x.abs() >= dir.y.abs() {
            if dir.x < 0.0 {
                r.x -= ATTACK_REACH;
            } else {
                r.x += body.w;
            }
            r.w = ATTACK_REACH;
        } else {
            if dir.y < 0.0 {
                r.y -= ATTACK_REACH;
            } else {
                r.y += body.h;
            }
            r.h = ATTACK_REACH;
        }
        r
    }

    /// Deterministic per-ram pace variance (0.9..1.1) so the horde doesn't
    /// march in lockstep.
    fn speed_scale(id: u64) -> f32 {
        0.9 + (id % 5) as f32 * 0.05
    }

    /// Ease `current` toward `target`, moving at most `max_delta`.
    fn move_toward(current: Vec2, target: Vec2, max_delta: f32) -> Vec2 {
        let delta = target - current;
        let dist = delta.length();
        if dist <= max_delta || dist < f32::EPSILON {
            target
        } else {
            current + delta / dist * max_delta
        }
    }

    /// Boids-style separation steering away from other ram heads; the
    /// collision system remains the hard backstop for actual overlaps.
    fn separation(center: Vec2, self_id: u64, ram_centers: &[(u64, Vec2)]) -> Vec2 {
        let mut steer = Vec2::ZERO;
        for (id, other) in ram_centers {
            if *id == self_id {
                continue;
            }
            let diff = center - *other;
            let dist = diff.length();
            if dist > 0.001 && dist < SEPARATION_RADIUS {
                steer += diff / dist * (1.0 - dist / SEPARATION_RADIUS);
            }
        }
        steer
    }

    pub fn new(store: Rc<EStore>, bus: Rc<EventBus>) -> Self {
        Self {
            store,
            bus,
            states: HashMap::new(),
        }
    }

    /// Returns the id and bounding rect of the nearest living hero or peon to
    /// `origin`, or None when no target is alive.
    fn nearest_target(&self, origin: Vec2) -> Option<(u64, Rect)> {
        let mut best: Option<(u64, Rect, f32)> = None;

        if let Some(knight) = self
            .store
            .first::<PlayerOne>()
            .and_then(|player| self.store.get_child::<Knight>(&player))
        {
            let alive = self
                .store
                .get_child::<HeroStats>(&knight)
                .map(|s| s.hp > 0)
                .unwrap_or(true);
            if alive {
                if let Some(anim) = self.store.get_child::<Animation>(&knight) {
                    let rect = anim.rect();
                    let dist = (rect.center() - origin).length();
                    best = Some((knight.entity_ref().id(), rect, dist));
                }
            }
        }

        for peon in self.store.all::<Peon>() {
            let alive = self
                .store
                .get_child::<PeonStats>(&peon)
                .map(|s| s.hp > 0)
                .unwrap_or(true);
            if !alive {
                continue;
            }
            if let Some(anim) = self.store.get_child::<Animation>(&peon) {
                let rect = anim.rect();
                let dist = (rect.center() - origin).length();
                if best.as_ref().is_none_or(|(_, _, d)| dist < *d) {
                    best = Some((peon.entity_ref().id(), rect, dist));
                }
            }
        }

        best.map(|(id, rect, _)| (id, rect))
    }

    /// Returns Some(target_id) when `target`'s rect overlaps the ram's forward
    /// attack rect (simple rect overlap, no pixel sampling); None otherwise.
    fn target_in_front(&self, body: Rect, facing: Vec2, target: Option<(u64, Rect)>) -> Option<u64> {
        let (id, rect) = target?;
        let forward = Self::forward_rect(body, facing);
        if forward.overlaps(&rect) {
            Some(id)
        } else {
            None
        }
    }
}

impl System for RamHeadAiSystem {
    fn update(&mut self, ctx: &mut Context) {
        let dt = ctx.dt;

        let enemies: Vec<(u64, EntityRef, Rect)> = self
            .store
            .all::<RamHead>()
            .filter_map(|enemy| {
                let id = enemy.entity_ref().id();
                self.store
                    .get_child::<Animation>(&enemy)
                    .map(|anim| (id, anim.entity_ref(), anim.rect()))
            })
            .collect();

        let ram_centers: Vec<(u64, Vec2)> = enemies
            .iter()
            .map(|(id, _, rect)| (*id, rect.center()))
            .collect();

        let mut anim_writes: Vec<AnimWrite> = Vec::new();
        let mut area_writes: Vec<AttackRectWrite> = Vec::new();

        for (enemy_id, anim_ref, body) in &enemies {
            let mut brain = self.states.get(enemy_id).cloned().unwrap_or_default();

            brain.attack_cooldown = (brain.attack_cooldown - dt).max(0.0);
            brain.charge_cooldown = (brain.charge_cooldown - dt).max(0.0);

            let center = body.center();

            let target = self.nearest_target(center);
            let mut facing = match &target {
                Some((_, rect)) => {
                    let delta = rect.center() - center;
                    if delta.length() > 0.001 {
                        delta.normalize()
                    } else {
                        brain.facing
                    }
                }
                None => vec2(-1.0, 0.0),
            };

            // While a swing is active, keep tracking the victim we committed to.
            if brain.attacking {
                if let Some(tid) = brain.attack_target {
                    let victim_rect = self
                        .store
                        .get_by_id::<Knight>(tid)
                        .and_then(|k| self.store.get_child::<Animation>(&k).map(|a| a.rect()))
                        .or_else(|| {
                            self.store
                                .get_by_id::<Peon>(tid)
                                .and_then(|p| self.store.get_child::<Animation>(&p).map(|a| a.rect()))
                        });
                    if let Some(rect) = victim_rect {
                        let delta = rect.center() - center;
                        if delta.length() > 0.001 {
                            facing = delta.normalize();
                        }
                    }
                }
            }
            brain.facing = facing;

            // Tick down the active attack, closing it out when it expires.
            let mut attacking = brain.attacking;
            if brain.attack_timer > 0.0 {
                brain.attack_timer = (brain.attack_timer - dt).max(0.0);
                if brain.attack_timer <= 0.0 {
                    attacking = false;
                    brain.attacking = false;
                    brain.attack_target = None;
                }
            }

            // Movement mode machine: pick this frame's desired velocity.
            let max_speed = MOVE_SPEED * Self::speed_scale(*enemy_id);
            let mut desired = Vec2::ZERO;
            let mut accel = ACCEL;
            let mut tint = NORMAL_TINT;

            match brain.mode {
                RamMode::Stalk => {
                    let seek = match &target {
                        Some((_, rect)) => {
                            let delta = rect.center() - center;
                            if delta.length() > 0.001 {
                                delta.normalize()
                            } else {
                                facing
                            }
                        }
                        None => vec2(-1.0, 0.0),
                    };
                    desired = seek * max_speed
                        + Self::separation(center, *enemy_id, &ram_centers) * SEPARATION_WEIGHT;
                    let cap = max_speed * 1.5;
                    if desired.length() > cap {
                        desired = desired.normalize() * cap;
                    }

                    let in_charge_range = target
                        .as_ref()
                        .map(|(_, rect)| (rect.center() - center).length() <= CHARGE_TRIGGER_RANGE)
                        .unwrap_or(false);
                    if in_charge_range && brain.charge_cooldown <= 0.0 && !attacking {
                        brain.mode = RamMode::Windup;
                        brain.mode_timer = WINDUP_SECS;
                    }
                }
                RamMode::Windup => {
                    // Telegraph: brake hard and flash red while lining up.
                    tint = WINDUP_TINT;
                    brain.mode_timer -= dt;
                    if brain.mode_timer <= 0.0 {
                        brain.mode = RamMode::Charge;
                        brain.mode_timer = CHARGE_MAX_SECS;
                        brain.charge_dir = match &target {
                            Some((_, rect)) => {
                                let delta = rect.center() - center;
                                if delta.length() > 0.001 {
                                    delta.normalize()
                                } else {
                                    facing
                                }
                            }
                            None => facing,
                        };
                    }
                }
                RamMode::Charge => {
                    accel = CHARGE_ACCEL;
                    desired = brain.charge_dir * CHARGE_SPEED;
                    facing = brain.charge_dir;
                    brain.facing = facing;
                    brain.mode_timer -= dt;
                    if brain.mode_timer <= 0.0 {
                        brain.mode = RamMode::Recover;
                        brain.mode_timer = RECOVER_SECS;
                        brain.charge_cooldown = CHARGE_COOLDOWN;
                    }
                }
                RamMode::Recover => {
                    desired = brain.charge_dir * max_speed * 0.3;
                    brain.mode_timer -= dt;
                    if brain.mode_timer <= 0.0 {
                        brain.mode = RamMode::Stalk;
                    }
                }
            }

            // Melee attack: allowed while stalking (adjacent prey) or mid-charge
            // (the charge connects).
            let can_attack = matches!(brain.mode, RamMode::Stalk | RamMode::Charge)
                && brain.attack_cooldown <= 0.0
                && !attacking;
            if can_attack {
                if let Some(target_id) = self.target_in_front(*body, facing, target) {
                    brain.attacking = true;
                    brain.attack_timer = ATTACK_DURATION;
                    brain.attack_step = 0;
                    brain.frame_elapsed = 0.0;
                    brain.attack_cooldown = ATTACK_COOLDOWN;
                    brain.attack_target = Some(target_id);
                    attacking = true;

                    let strength = self
                        .store
                        .get_by_id::<RamHead>(*enemy_id)
                        .and_then(|enemy| self.store.get_child::<EnemyStats>(&enemy))
                        .map(|stats| stats.strength)
                        .unwrap_or(1)
                        .max(1);

                    self.bus.fire(&EnemyAttackEvent {
                        target: target_id,
                        damage: strength,
                    });
                    self.bus.fire(&HitEvent {
                        victim: target_id,
                        attacker: *enemy_id,
                    });

                    if brain.mode == RamMode::Charge {
                        brain.mode = RamMode::Recover;
                        brain.mode_timer = RECOVER_SECS;
                        brain.charge_cooldown = CHARGE_COOLDOWN;
                    }
                }
            }

            // Ease the velocity toward the desired velocity and integrate.
            brain.vel = Self::move_toward(brain.vel, desired, accel * dt);
            let mut position = vec2(body.x + brain.vel.x * dt, body.y + brain.vel.y * dt);
            position.y = position.y.clamp(Y_BOUND_MIN, Y_BOUND_MAX);

            // Animation frames: attack frames while attacking, walk frames
            // while moving, hold the first walk frame when (nearly) stopped.
            let (frame, flip_x) = if attacking {
                let frame = ATTACK_FRAMES[brain.attack_step];
                brain.frame_elapsed += dt;
                if brain.frame_elapsed >= ATTACK_FRAME_DURATION {
                    brain.frame_elapsed = 0.0;
                    brain.attack_step = (brain.attack_step + 1) % ATTACK_FRAMES.len();
                }
                (frame, facing.x < 0.0)
            } else if brain.vel.length() > WALK_ANIM_MIN_SPEED {
                let frame = WALK_FRAMES[brain.walk_step];
                brain.frame_elapsed += dt;
                if brain.frame_elapsed >= FRAME_DURATION {
                    brain.frame_elapsed = 0.0;
                    brain.walk_step = (brain.walk_step + 1) % WALK_FRAMES.len();
                }
                (frame, facing.x < 0.0)
            } else {
                (WALK_FRAMES[0], facing.x < 0.0)
            };

            anim_writes.push(AnimWrite {
                anim_ref: *anim_ref,
                position,
                frame,
                flip_x,
                tint,
            });

            if let Some(area_ref) = self
                .store
                .get_by_id::<RamHead>(*enemy_id)
                .and_then(|enemy| self.store.get_child::<AttackRect>(&enemy))
                .map(|area| area.entity_ref())
            {
                let rects = if attacking {
                    vec![Self::forward_rect(*body, facing)]
                } else {
                    Vec::new()
                };
                area_writes.push(AttackRectWrite {
                    area_ref,
                    rects,
                    visible: attacking,
                });
            }

            self.states.insert(*enemy_id, brain);
        }

        for w in anim_writes {
            let AnimWrite {
                anim_ref,
                position,
                frame,
                flip_x,
                tint,
            } = w;
            self.store.update::<Animation, _>(&anim_ref, |a| {
                a.position = position;
                a.current_frame = frame;
                a.flip_x = flip_x;
                a.tint = tint;
            });
        }

        for w in area_writes {
            let AttackRectWrite {
                area_ref,
                rects,
                visible,
            } = w;
            self.store.update::<AttackRect, _>(&area_ref, |area| {
                area.rects = rects;
                area.visible = visible;
            });
        }
    }
}
