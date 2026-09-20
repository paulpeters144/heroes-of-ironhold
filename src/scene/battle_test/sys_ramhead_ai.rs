use crate::entity::enemy::{EnemyStats, RamHead, ATTACK_FRAMES, IDLE_FRAME, WALK_FRAMES};
use crate::entity::factory_enemy::{COLLISION_HEIGHT_SCALE, COLLISION_WIDTH_SCALE, FRAME_SIZE};
use crate::entity::knight::Knight;
use crate::entity::PlayerOne;
use crate::events::{EnemyAttackEvent, HitEvent};
use crate::prelude::*;
use crate::util::{did_attack, image_data_for};
use macroquad::prelude::{vec2, Color, Image, Rect, Texture2D, Vec2};
use pico_entity_store::prelude::EntityRef;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;

// Chase movement: walk speed and walk frame timing.
const MOVE_SPEED: f32 = 70.0;
const MOVE_SPEED_VERTICAL: f32 = MOVE_SPEED * 0.75;
const FRAME_DURATION: f32 = 0.14;

// When the knight gets this close, the ram stops chasing and begins its charge.
const WINDUP_RANGE: f32 = 150.0;

// The ram telegraphs its charge (locked aim + red flash) before lunging, so a
// player can react. Below half hp it telegraphs and recovers faster.
const WINDUP_SECS: f32 = 0.4;
const WINDUP_ADVANCE: f32 = 40.0;
const CHARGE_SECS: f32 = 0.30;
const CHARGE_SPEED: f32 = 200.0;
const CHARGE_FRAME_DURATION: f32 = 0.06;
const CHARGE_REACH: f32 = 26.0;
const RECOVER_SECS: f32 = 0.45;
const COOLDOWN_SECS: f32 = 0.55;

// A hit cancels the ram's attack and staggers it briefly before it can re-engage.
const HIT_STAGGER_SECS: f32 = 0.35;

// How hard the cluster pushes overlapping ram heads apart so they don't jam.
const SEPARATION_ITERATIONS: usize = 8;
// A blocked ram head slides sideways within this range of a neighbor.
const CONTACT_RADIUS: f32 = 40.0;
const SLIDE_SPEED: f32 = 60.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Chase,
    Windup,
    Charge,
    Recover,
}

/// Per-ram AI state machine. Kept on the system (not a component) keyed by the
/// ram's entity id so any number of ram heads run independently.
#[derive(Clone)]
struct AiState {
    state: State,
    state_timer: f32,
    cooldown: f32,
    frame_elapsed: f32,
    walk_step: usize,
    attack_step: usize,
    charge_dir: Vec2,
    hit_this_charge: bool,
}

impl Default for AiState {
    fn default() -> Self {
        AiState {
            state: State::Chase,
            state_timer: 0.0,
            cooldown: 0.0,
            frame_elapsed: 0.0,
            walk_step: 0,
            attack_step: 0,
            charge_dir: Vec2::ZERO,
            hit_this_charge: false,
        }
    }
}

impl AiState {
    /// Cancel any in-progress attack and stagger back into chase. Called when the
    /// ram is hit so the player's strike visibly interrupts it.
    fn interrupt(&mut self) {
        self.state = State::Chase;
        self.state_timer = 0.0;
        self.cooldown = HIT_STAGGER_SECS;
        self.frame_elapsed = 0.0;
        self.walk_step = 0;
        self.attack_step = 0;
        self.charge_dir = Vec2::ZERO;
        self.hit_this_charge = false;
    }
}

struct AnimWrite {
    enemy_id: u64,
    anim_ref: EntityRef,
    frame: usize,
    flip_x: bool,
    tint: Color,
}

struct AttackRectWrite {
    area_ref: EntityRef,
    rects: Vec<Rect>,
    visible: bool,
}

/// One ram head's movement this frame, before collision resolution.
struct Plan {
    enemy_id: u64,
    start: Vec2,
    pos: Vec2,
    seek: Vec2,
    chasing: bool,
}

/// Push two bodies out of overlap along the axis of least penetration.
/// When `move_b` is false only `a` moves (used for the immovable hero).
fn separate(a: &mut Vec2, b: &mut Vec2, half_a: Vec2, half_b: Vec2, move_b: bool) {
    let delta = *a - *b;
    let overlap_x = half_a.x + half_b.x - delta.x.abs();
    let overlap_y = half_a.y + half_b.y - delta.y.abs();
    if overlap_x <= 0.0 || overlap_y <= 0.0 {
        return;
    }
    if overlap_x < overlap_y {
        let dir = if delta.x < 0.0 { -1.0 } else { 1.0 };
        if move_b {
            let push = overlap_x * 0.5;
            a.x += push * dir;
            b.x -= push * dir;
        } else {
            a.x += overlap_x * dir;
        }
    } else {
        let dir = if delta.y < 0.0 { -1.0 } else { 1.0 };
        if move_b {
            let push = overlap_y * 0.5;
            a.y += push * dir;
            b.y -= push * dir;
        } else {
            a.y += overlap_y * dir;
        }
    }
}

pub struct RamHeadAiSystem {
    store: Rc<EStore>,
    bus: Rc<EventBus>,
    states: HashMap<u64, AiState>,
    sheet_cache: Option<(Texture2D, Image)>,
    hit_queue: Rc<RefCell<VecDeque<u64>>>,
    _subs: Rc<SubCollection>,
}

impl RamHeadAiSystem {
    /// Returns the charge attack rectangle: a CHARGE_REACH-deep strip just in
    /// front of `body` toward `dir`, so the attack sits ahead of the ram and
    /// doesn't cover its own body.
    fn charge_rect(body: Rect, dir: Vec2) -> Rect {
        let mut r = body;
        if dir.x.abs() >= dir.y.abs() {
            if dir.x < 0.0 {
                r.x -= CHARGE_REACH;
            } else {
                r.x += body.w;
            }
            r.w = CHARGE_REACH;
        } else {
            if dir.y < 0.0 {
                r.y -= CHARGE_REACH;
            } else {
                r.y += body.h;
            }
            r.h = CHARGE_REACH;
        }
        r
    }

    pub fn new(store: Rc<EStore>, bus: Rc<EventBus>) -> Self {
        let hit_queue = Rc::new(RefCell::new(VecDeque::new()));
        let subs = Rc::new(SubCollection::new());
        let queue_for_handler = hit_queue.clone();
        subs.on::<HitEvent>(&bus, move |event: &HitEvent| {
            queue_for_handler.borrow_mut().push_back(event.victim);
        });
        Self {
            store,
            bus,
            states: HashMap::new(),
            sheet_cache: None,
            hit_queue,
            _subs: subs,
        }
    }
}

impl System for RamHeadAiSystem {
    fn update(&mut self, ctx: &mut Context) {
        let dt = ctx.dt;

        // The knight is the lone player; snapshot its position and CPU image
        // data once for all ram heads this frame.
        let knight = self
            .store
            .first::<PlayerOne>()
            .and_then(|player| self.store.get_child::<Knight>(&player))
            .and_then(|knight| self.store.get_child::<Animation>(&knight))
            .map(|anim| {
                let data = image_data_for(&anim, &mut self.sheet_cache);
                (anim.position, data)
            });

        // Collect every ram's anim ref up front so no store read lock lives
        // across the state-machine mutation or the write-back pass below.
        let enemies: Vec<(u64, EntityRef)> = self
            .store
            .all::<RamHead>()
            .filter_map(|enemy| {
                let id = enemy.entity_ref().id();
                self.store
                    .get_child::<Animation>(&enemy)
                    .map(|anim| (id, anim.entity_ref()))
            })
            .collect();

        let mut anim_writes: Vec<AnimWrite> = Vec::new();
        let mut area_writes: Vec<AttackRectWrite> = Vec::new();
        let mut plans: Vec<Plan> = Vec::new();

        let interrupted: HashSet<u64> = self.hit_queue.borrow_mut().drain(..).collect();

        for (enemy_id, anim_ref) in &enemies {
            let state = self.states.entry(*enemy_id).or_default();

            if interrupted.contains(enemy_id) {
                state.interrupt();
            }

            let Some(enemy_pos) = self
                .store
                .get_by_id::<Animation>(anim_ref.id())
                .map(|a| a.position)
            else {
                continue;
            };

            let (knight_pos, knight_data) = match &knight {
                Some((pos, data)) => (*pos, Some(data)),
                None => {
                    self.store.update::<Animation, _>(anim_ref, |animation| {
                        animation.current_frame = IDLE_FRAME;
                    });
                    continue;
                }
            };

            let delta = knight_pos - enemy_pos;
            let distance = delta.length();

            let enraged = self
                .store
                .get_by_id::<RamHead>(*enemy_id)
                .and_then(|enemy| self.store.get_child::<EnemyStats>(&enemy))
                .map(|stats| stats.hp * 2 <= stats.max_hp)
                .unwrap_or(false);

            let cooldown_secs = if enraged {
                COOLDOWN_SECS * 0.5
            } else {
                COOLDOWN_SECS
            };
            let windup_secs = if enraged {
                WINDUP_SECS * 0.6
            } else {
                WINDUP_SECS
            };
            let charge_speed = if enraged {
                CHARGE_SPEED * 1.15
            } else {
                CHARGE_SPEED
            };

            state.cooldown = (state.cooldown - dt).max(0.0);

            match state.state {
                State::Chase => {
                    if distance > 0.0 && state.cooldown <= 0.0 && distance <= WINDUP_RANGE {
                        state.state = State::Windup;
                        state.state_timer = windup_secs;
                        state.charge_dir = delta / distance;
                        state.frame_elapsed = 0.0;
                        state.attack_step = 0;
                    }
                }
                State::Windup => {
                    state.state_timer -= dt;
                    if state.state_timer <= 0.0 {
                        state.state = State::Charge;
                        state.state_timer = CHARGE_SECS;
                        state.hit_this_charge = false;
                        state.frame_elapsed = 0.0;
                        state.attack_step = 0;
                    }
                }
                State::Charge => {
                    state.state_timer -= dt;
                    if state.state_timer <= 0.0 {
                        state.state = State::Recover;
                        state.state_timer = RECOVER_SECS;
                    }
                }
                State::Recover => {
                    state.state_timer -= dt;
                    if state.state_timer <= 0.0 {
                        state.state = State::Chase;
                        state.cooldown = cooldown_secs;
                    }
                }
            }

            let mut new_pos = enemy_pos;
            let (frame, flip_x, tint) = match state.state {
                State::Chase => {
                    if distance > 0.0 {
                        let dir = delta / distance;
                        new_pos += vec2(dir.x * MOVE_SPEED, dir.y * MOVE_SPEED_VERTICAL) * dt;
                    }
                    state.frame_elapsed += dt;
                    let frame = WALK_FRAMES[state.walk_step];
                    if state.frame_elapsed >= FRAME_DURATION {
                        state.frame_elapsed = 0.0;
                        state.walk_step = (state.walk_step + 1) % WALK_FRAMES.len();
                    }
                    (frame, delta.x < 0.0, Color::new(1.0, 1.0, 1.0, 1.0))
                }
                State::Windup => {
                    new_pos += state.charge_dir * WINDUP_ADVANCE * dt;
                    (
                        ATTACK_FRAMES[0],
                        state.charge_dir.x < 0.0,
                        Color::new(1.0, 0.5, 0.5, 1.0),
                    )
                }
                State::Charge => {
                    new_pos += state.charge_dir * charge_speed * dt;
                    state.frame_elapsed += dt;
                    let frame = ATTACK_FRAMES[state.attack_step];
                    if state.frame_elapsed >= CHARGE_FRAME_DURATION {
                        state.frame_elapsed = 0.0;
                        state.attack_step = (state.attack_step + 1) % ATTACK_FRAMES.len();
                    }
                    (
                        frame,
                        state.charge_dir.x < 0.0,
                        Color::new(1.0, 1.0, 1.0, 1.0),
                    )
                }
                State::Recover => (
                    ATTACK_FRAMES[ATTACK_FRAMES.len() - 1],
                    state.charge_dir.x < 0.0,
                    Color::new(1.0, 1.0, 1.0, 1.0),
                ),
            };

            anim_writes.push(AnimWrite {
                enemy_id: *enemy_id,
                anim_ref: *anim_ref,
                frame,
                flip_x,
                tint,
            });
            let seek = if distance > 0.0 {
                delta / distance
            } else {
                Vec2::ZERO
            };
            plans.push(Plan {
                enemy_id: *enemy_id,
                start: enemy_pos,
                pos: new_pos,
                seek,
                chasing: state.state == State::Chase,
            });

            let attacking = state.state == State::Charge;
            let charge_rect = self
                .store
                .get_by_id::<Animation>(anim_ref.id())
                .map(|anim| Self::charge_rect(anim.rect(), state.charge_dir))
                .unwrap_or(Rect::new(0.0, 0.0, 0.0, 0.0));

            if let Some(area_ref) = self
                .store
                .get_by_id::<RamHead>(*enemy_id)
                .and_then(|enemy| self.store.get_child::<AttackRect>(&enemy))
                .map(|area| area.entity_ref())
            {
                let rects = if attacking {
                    vec![charge_rect]
                } else {
                    Vec::new()
                };
                area_writes.push(AttackRectWrite {
                    area_ref,
                    rects,
                    visible: attacking,
                });
            }

            // A charge deals its damage once, on first contact with the knight.
            if attacking && !state.hit_this_charge {
                if let Some(data) = knight_data {
                    if did_attack(charge_rect, data) {
                        let target = self
                            .store
                            .first::<PlayerOne>()
                            .and_then(|player| self.store.get_child::<Knight>(&player))
                            .map(|knight| knight.entity_ref().id());
                        if let Some(target) = target {
                            let damage = self
                                .store
                                .get_by_id::<RamHead>(*enemy_id)
                                .and_then(|enemy| self.store.get_child::<EnemyStats>(&enemy))
                                .map(|stats| stats.strength)
                                .unwrap_or(1)
                                .max(1);
                            self.bus.fire(&EnemyAttackEvent { target, damage });
                            self.bus.fire(&HitEvent {
                                victim: target,
                                attacker: *enemy_id,
                            });
                        }
                        state.hit_this_charge = true;
                    }
                }
            }
        }

        // Ram heads and the hero share the same collision extents.
        let half = vec2(
            FRAME_SIZE * COLLISION_WIDTH_SCALE,
            FRAME_SIZE * COLLISION_HEIGHT_SCALE,
        ) * 0.5;
        let knight_pos = knight.as_ref().map(|(pos, _)| *pos);

        // A chasing ram head whose path is blocked slips sideways toward the
        // knight so it walks around the pile instead of pushing into it.
        for i in 0..plans.len() {
            if !plans[i].chasing || plans[i].seek == Vec2::ZERO {
                continue;
            }
            let achieved = plans[i].pos - plans[i].start;
            if achieved.dot(plans[i].seek) >= MOVE_SPEED * dt * 0.25 {
                continue;
            }
            let mut slide = Vec2::ZERO;
            for j in 0..plans.len() {
                if i == j {
                    continue;
                }
                let delta = plans[i].pos - plans[j].pos;
                let dist = delta.length();
                if dist <= 0.001 || dist >= CONTACT_RADIUS {
                    continue;
                }
                let dir = delta / dist;
                let perp = vec2(-dir.y, dir.x);
                let sign = if perp.dot(plans[i].seek) < 0.0 {
                    -1.0
                } else {
                    1.0
                };
                slide += perp * sign;
            }
            if slide.length_squared() > 0.0 {
                plans[i].pos += slide.normalize() * SLIDE_SPEED * dt;
            }
        }

        // Push overlapping ram heads apart along the axis of least penetration
        // so a dense cluster spreads out instead of locking in place.
        for _ in 0..SEPARATION_ITERATIONS {
            for i in 0..plans.len() {
                for j in (i + 1)..plans.len() {
                    let (head, tail) = plans.split_at_mut(j);
                    separate(&mut head[i].pos, &mut tail[0].pos, half, half, true);
                }
                if let Some(kp) = knight_pos {
                    let mut knight = kp;
                    separate(&mut plans[i].pos, &mut knight, half, half, false);
                }
            }
        }

        let resolved: HashMap<u64, Vec2> =
            plans.iter().map(|plan| (plan.enemy_id, plan.pos)).collect();

        for w in anim_writes {
            let AnimWrite {
                enemy_id,
                anim_ref,
                frame,
                flip_x,
                tint,
            } = w;
            let Some(pos) = resolved.get(&enemy_id).copied() else {
                continue;
            };
            self.store.update::<Animation, _>(&anim_ref, |animation| {
                animation.position.x = pos.x.round();
                animation.position.y = pos.y.round();
                animation.current_frame = frame;
                animation.flip_x = flip_x;
                animation.tint = tint;
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
