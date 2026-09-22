use crate::entity::enemy::{EnemyStats, RamHead, ATTACK_FRAMES, WALK_FRAMES};
use crate::entity::knight::Knight;
use crate::entity::{HeroStats, Peon, PeonStats, PlayerOne};
use crate::events::{EnemyAttackEvent, HitEvent};
use crate::prelude::*;
use macroquad::prelude::{vec2, Color, Rect, Vec2};
use pico_entity_store::prelude::EntityRef;
use std::collections::HashMap;
use std::rc::Rc;

// Always-forward walk speed and walk-frame timing (horizontal faster than vertical).
const MOVE_SPEED: f32 = 60.0;
const MOVE_SPEED_VERTICAL: f32 = MOVE_SPEED * 0.75;
const FRAME_DURATION: f32 = 0.14;

// Melee attack: how far ahead the hit strip reaches and its timing.
const ATTACK_REACH: f32 = 26.0;
const ATTACK_DURATION: f32 = 0.30;
const ATTACK_FRAME_DURATION: f32 = 0.06;
const ATTACK_COOLDOWN: f32 = 0.55;

// Forward-obstacle probe: how far ahead to look and how wide a corridor counts
// as "blocked"; the sideways speed used to walk around a blocker.
const PROBE_DIST: f32 = 64.0;
const OBSTACLE_CLEARANCE: f32 = 32.0;
const STEER_SPEED: f32 = 60.0;

// Keep ram heads within the map's vertical bounds.
const Y_BOUND_MIN: f32 = 40.0;
const Y_BOUND_MAX: f32 = 360.0;
const BOUND_MARGIN: f32 = 24.0;

/// Per-ram walk/attack state. An always-forward walk plus a brief melee attack;
/// no state in which the ram stops moving.
#[derive(Clone)]
struct RamBrain {
    facing: Vec2, // unit direction the ram walks this frame; defaults to left (-1, 0) when no target is visible
    walk_step: usize, // index into WALK_FRAMES
    frame_elapsed: f32, // walk/attack-frame animation accumulator
    attack_cooldown: f32, // seconds until the next attack is allowed
    attacking: bool, // true while a melee attack is active (opens AttackRect)
    attack_timer: f32, // remaining time of the active attack; plays ATTACK_FRAMES
    attack_step: usize, // index into ATTACK_FRAMES
    attack_target: Option<u64>, // entity id of the hero/peon being attacked
}

impl Default for RamBrain {
    fn default() -> Self {
        RamBrain {
            facing: vec2(-1.0, 0.0),
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

/// Something blocking the ram's forward path that it must walk around.
#[derive(Clone, Copy)]
enum Obstacle {
    Ram(Vec2), // another ram head's position; steer sideways to clear it
    Boundary,  // top/bottom map boundary; keep moving but stop vertical advance
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
                if best.as_ref().map_or(true, |(_, _, d)| dist < *d) {
                    best = Some((peon.entity_ref().id(), rect, dist));
                }
            }
        }

        best.map(|(id, rect, _)| (id, rect))
    }

    /// Returns the obstacle (another ram head or the vertical map boundary)
    /// immediately in front of `origin` along `facing`, excluding `self_id`.
    fn obstacle_ahead(&self, origin: Vec2, facing: Vec2, self_id: u64) -> Option<Obstacle> {
        let mut closest: Option<(Vec2, f32)> = None;

        for enemy in self.store.all::<RamHead>() {
            if enemy.entity_ref().id() == self_id {
                continue;
            }
            let Some(anim) = self.store.get_child::<Animation>(&enemy) else {
                continue;
            };
            let center = anim.rect().center();
            let delta = center - origin;
            let along = delta.dot(facing);
            if along <= 0.0 || along > PROBE_DIST {
                continue;
            }
            let lateral = (delta - facing * along).length();
            if lateral > OBSTACLE_CLEARANCE {
                continue;
            }
            if closest.as_ref().map_or(true, |(_, d)| along < *d) {
                closest = Some((center, along));
            }
        }

        if let Some((pos, _)) = closest {
            return Some(Obstacle::Ram(pos));
        }

        if facing.y < -0.01 && origin.y - Y_BOUND_MIN < BOUND_MARGIN {
            return Some(Obstacle::Boundary);
        }
        if facing.y > 0.01 && Y_BOUND_MAX - origin.y < BOUND_MARGIN {
            return Some(Obstacle::Boundary);
        }

        None
    }

    /// Computes this frame's forward walk velocity, steering sideways to clear
    /// `obstacle` while continuing to advance along `facing`. Speed is
    /// axis-scaled: `MOVE_SPEED` horizontally, `MOVE_SPEED_VERTICAL` vertically.
    fn steer(&self, origin: Vec2, facing: Vec2, obstacle: Option<Obstacle>) -> Vec2 {
        let mut velocity = vec2(facing.x * MOVE_SPEED, facing.y * MOVE_SPEED_VERTICAL);
        match obstacle {
            None => {}
            Some(Obstacle::Boundary) => {
                velocity.y = 0.0;
            }
            Some(Obstacle::Ram(pos)) => {
                let perp = vec2(-facing.y, facing.x);
                let side = if perp.dot(pos - origin) > 0.0 {
                    -1.0
                } else {
                    1.0
                };
                velocity += perp * side * STEER_SPEED;
            }
        }
        velocity
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

        let mut anim_writes: Vec<AnimWrite> = Vec::new();
        let mut area_writes: Vec<AttackRectWrite> = Vec::new();

        for (enemy_id, anim_ref, body) in &enemies {
            let mut brain = self.states.get(enemy_id).cloned().unwrap_or_default();

            brain.attack_cooldown = (brain.attack_cooldown - dt).max(0.0);

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

            let obstacle = self.obstacle_ahead(center, facing, *enemy_id);
            let velocity = self.steer(center, facing, obstacle);

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

            // Start a melee attack when a target is directly in front and the
            // cooldown has elapsed.
            if brain.attack_cooldown <= 0.0 && !attacking {
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
                }
            }

            // Animation frames: attack frames while attacking, else walk frames.
            let (frame, flip_x) = if attacking {
                let frame = ATTACK_FRAMES[brain.attack_step];
                brain.frame_elapsed += dt;
                if brain.frame_elapsed >= ATTACK_FRAME_DURATION {
                    brain.frame_elapsed = 0.0;
                    brain.attack_step = (brain.attack_step + 1) % ATTACK_FRAMES.len();
                }
                (frame, facing.x < 0.0)
            } else {
                let frame = WALK_FRAMES[brain.walk_step];
                brain.frame_elapsed += dt;
                if brain.frame_elapsed >= FRAME_DURATION {
                    brain.frame_elapsed = 0.0;
                    brain.walk_step = (brain.walk_step + 1) % WALK_FRAMES.len();
                }
                (frame, facing.x < 0.0)
            };

            let mut position = vec2(body.x + velocity.x * dt, body.y + velocity.y * dt);
            position.y = position.y.clamp(Y_BOUND_MIN, Y_BOUND_MAX);

            anim_writes.push(AnimWrite {
                anim_ref: *anim_ref,
                position,
                frame,
                flip_x,
                tint: Color::new(1.0, 1.0, 1.0, 1.0),
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
