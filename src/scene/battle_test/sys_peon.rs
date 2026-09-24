use crate::entity::enemy::{EnemyStats, RamHead};
use crate::entity::factory_peon::PEON_FRAME_SIZE;
use crate::entity::peon::{
    Peon, PeonStats, PEON_ATTACK_HIT1_FRAMES, PEON_ATTACK_HIT2_FRAMES, PEON_WALK_FRAMES,
};
use crate::entity::{
    AttackRect, Consecration, HealthBar, ImpactFrame, Knight, PeonCfg, PeonFactory,
    PeonSpawnZone, PlayerOne,
};
use crate::events::{
    AttackEvent, EnemyAttackEvent, EnemyDeathEvent, HealthChangeEvent, HitEvent, PeonDeathEvent,
    SpawnPeonSquadEvent,
};
use crate::prelude::*;
use crate::{images, Assets};
use macroquad::prelude::{
    draw_rectangle, draw_texture_ex, vec2, Color, DrawTextureParams, Rect, Texture2D, Vec2,
};
use macroquad::rand::gen_range;
use pico_entity_store::store::IntoChild;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

const MOVE_SPEED: f32 = 60.0;
const ACCEL: f32 = 500.0;
const WALK_FRAME_DURATION: f32 = 0.12;
const WALK_ANIM_MIN_SPEED: f32 = 8.0;
const ATTACK_RANGE: f32 = 55.0;
const ENGAGE_DIST: f32 = 40.0;
const CHASE_RANGE: f32 = 300.0;
const SLOW_RADIUS: f32 = 60.0;
const ATTACK_FRAME_DURATION: f32 = 0.08;
const RECOVER_SECS: f32 = 0.4;
const Y_BOUND_MIN: f32 = 40.0;
const Y_BOUND_MAX: f32 = 360.0;
const SEPARATION_RADIUS: f32 = 40.0;
const SEPARATION_WEIGHT: f32 = 130.0;
const FORMATION_X_OFFSET: f32 = 100.0;
const FORMATION_COLS: usize = 3;
const FORMATION_COL_SPACING: f32 = 42.0;
const FORMATION_ROW_SPACING: f32 = 36.0;
const SLOT_ARRIVE_RADIUS: f32 = 20.0;
const DESPAWN_MARGIN: f32 = 80.0;
const MAX_PAST_KNIGHT: f32 = 200.0;

const DEATH_DURATION: f32 = 0.8;
const DEATH_PARTICLE_COUNT: usize = 16;

const DUST_LIGHT: Color = Color::new(0.95, 0.95, 0.9, 1.0);
const DUST_MID: Color = Color::new(0.7, 0.7, 0.65, 1.0);
const DUST_DARK: Color = Color::new(0.4, 0.4, 0.38, 1.0);

fn mitigated_damage(raw: i32, armor: i32) -> i32 {
    (raw - armor).max(1)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PeonState {
    Walk,
    Return,
    AttackHit1,
    AttackHit2,
    Recover,
}

#[derive(Clone)]
struct PeonAiState {
    state: PeonState,
    state_timer: f32,
    frame_elapsed: f32,
    walk_step: usize,
    attack_step: usize,
    facing_right: bool,
    hit_this_attack: bool,
    vel: Vec2,
}

impl Default for PeonAiState {
    fn default() -> Self {
        PeonAiState {
            state: PeonState::Walk,
            state_timer: 0.0,
            frame_elapsed: 0.0,
            walk_step: 0,
            attack_step: 0,
            facing_right: true,
            hit_this_attack: false,
            vel: Vec2::ZERO,
        }
    }
}

struct AnimWrite {
    anim_ref: pico_entity_store::prelude::EntityRef,
    frame: usize,
    flip_x: bool,
    tint: Color,
    position: Option<Vec2>,
}

struct AttackRectWrite {
    area_ref: pico_entity_store::prelude::EntityRef,
    rects: Vec<Rect>,
    visible: bool,
}

struct DeathParticle {
    pos: Vec2,
    vel: Vec2,
    size: f32,
    delay: f32,
    color: Color,
}

struct DeathFx {
    texture: Texture2D,
    source: Rect,
    position: Vec2,
    dest_size: Vec2,
    flip_x: bool,
    age: f32,
    particles: Vec<DeathParticle>,
}

pub struct PeonSystem {
    store: Rc<EStore>,
    bus: Rc<EventBus>,
    map_w: f32,
    body: Texture2D,
    impact: Texture2D,
    squad_queue: Rc<RefCell<VecDeque<SpawnPeonSquadEvent>>>,
    states: HashMap<u64, PeonAiState>,
    attack_queue: Rc<RefCell<VecDeque<AttackEvent>>>,
    enemy_attack_queue: Rc<RefCell<VecDeque<EnemyAttackEvent>>>,
    death_queue: Rc<RefCell<VecDeque<PeonDeathEvent>>>,
    death_fx: Vec<DeathFx>,
    _subs: Rc<SubCollection>,
}

impl PeonSystem {
    pub fn new(store: Rc<EStore>, bus: Rc<EventBus>, map_w: f32, assets: &Assets) -> Self {
        let attack_queue = Rc::new(RefCell::new(VecDeque::new()));
        let enemy_attack_queue = Rc::new(RefCell::new(VecDeque::new()));
        let death_queue = Rc::new(RefCell::new(VecDeque::new()));
        let squad_queue = Rc::new(RefCell::new(VecDeque::new()));
        let subs = Rc::new(SubCollection::new());

        let aq = attack_queue.clone();
        subs.on::<AttackEvent>(&bus, move |event: &AttackEvent| {
            aq.borrow_mut().push_back(event.clone());
        });

        let eq = enemy_attack_queue.clone();
        subs.on::<EnemyAttackEvent>(&bus, move |event: &EnemyAttackEvent| {
            eq.borrow_mut().push_back(event.clone());
        });

        let dq = death_queue.clone();
        subs.on::<PeonDeathEvent>(&bus, move |event: &PeonDeathEvent| {
            dq.borrow_mut().push_back(event.clone());
        });

        let sq = squad_queue.clone();
        subs.on::<SpawnPeonSquadEvent>(&bus, move |event: &SpawnPeonSquadEvent| {
            sq.borrow_mut().push_back(event.clone());
        });

        Self {
            store,
            bus,
            map_w,
            body: assets.texture(images::Npc::HeroPeon),
            impact: assets.texture(images::Npc::HeroPeonHit),
            squad_queue,
            states: HashMap::new(),
            attack_queue,
            enemy_attack_queue,
            death_queue,
            death_fx: Vec::new(),
            _subs: subs,
        }
    }

    fn spawn_peon(&self) {
        let Some(rect) = self.store.first::<PeonSpawnZone>().map(|z| z.rect) else {
            return;
        };
        let mut parts = PeonFactory::create_peon(PeonCfg {
            body: self.body.clone(),
            impact: self.impact.clone(),
        });
        let x = rect.x + gen_range(0.0, rect.w);
        let y = rect.y + gen_range(0.0, rect.h);
        parts.body.position = vec2(x, y);
        self.store.add(
            parts.marker,
            &[
                parts.body.into_child(),
                parts.collision_circle.into_child(),
                HealthBar::default().into_child(),
                PeonStats::default().into_child(),
                parts.impact_frame.into_child(),
                AttackRect {
                    rects: Vec::new(),
                    visible: false,
                }
                .into_child(),
            ],
        );

        let peon_id = self
            .store
            .all::<Peon>()
            .map(|e| e.entity_ref().id())
            .last()
            .expect("peon just added");

        if let Some(impact_frame) = self
            .store
            .get_by_id::<Peon>(peon_id)
            .and_then(|e| self.store.get_child::<ImpactFrame>(&e))
        {
            self.store
                .add(impact_frame, &[parts.impact_image.into_child()]);
        }
    }

    fn attack_rect(body: Rect, facing_right: bool) -> Rect {
        let reach = 20.0;
        if facing_right {
            Rect::new(body.x + body.w, body.y, reach, body.h)
        } else {
            Rect::new(body.x - reach, body.y, reach, body.h)
        }
    }

    /// Deterministic per-peon pace variance (0.9..1.1) so the squad doesn't
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

    /// Stable warband slot for the peon's sorted-id index: a staggered grid
    /// anchored in front of the knight so each peon owns a spot instead of
    /// fighting over a single rally pixel.
    fn formation_slot(index: usize, knight_pos: Vec2) -> Vec2 {
        let row = index / FORMATION_COLS;
        let col = index % FORMATION_COLS;
        let centered = col as f32 - (FORMATION_COLS as f32 - 1.0) / 2.0;
        let stagger = if row.is_multiple_of(2) {
            0.0
        } else {
            FORMATION_COL_SPACING * 0.5
        };
        vec2(
            knight_pos.x + FORMATION_X_OFFSET + row as f32 * FORMATION_ROW_SPACING,
            (knight_pos.y + centered * FORMATION_COL_SPACING + stagger)
                .clamp(Y_BOUND_MIN + 16.0, Y_BOUND_MAX - 16.0),
        )
    }

    /// One frame of steering locomotion: seek `target` (point + stop distance)
    /// with arrival slowdown, add boids-style separation from other peons,
    /// ease the velocity toward the desired velocity, integrate, and pick the
    /// walk frame. Returns the new position and the walk frame to show.
    fn locomotion(
        state: &mut PeonAiState,
        peon_id: u64,
        peon_pos: Vec2,
        target: Option<(Vec2, f32)>,
        peon_centers: &[(u64, Vec2)],
        dt: f32,
    ) -> (Vec2, usize) {
        let max_speed = MOVE_SPEED * Self::speed_scale(peon_id);

        let mut desired = Vec2::ZERO;
        if let Some((point, stop_dist)) = target {
            let delta = point - peon_pos;
            let remaining = delta.length() - stop_dist;
            if remaining > 2.0 {
                let arrive = (remaining / SLOW_RADIUS).clamp(0.2, 1.0);
                desired = delta.normalize() * (max_speed * arrive);
            }
        }

        let center = vec2(
            peon_pos.x + PEON_FRAME_SIZE / 2.0,
            peon_pos.y + PEON_FRAME_SIZE / 2.0,
        );
        let mut separation = Vec2::ZERO;
        for (other_id, other_center) in peon_centers {
            if *other_id == peon_id {
                continue;
            }
            let diff = center - *other_center;
            let dist = diff.length();
            if dist > 0.001 && dist < SEPARATION_RADIUS {
                separation += diff / dist * (1.0 - dist / SEPARATION_RADIUS);
            }
        }
        desired += separation * SEPARATION_WEIGHT;

        let cap = max_speed * 1.6;
        if desired.length() > cap {
            desired = desired.normalize() * cap;
        }

        state.vel = Self::move_toward(state.vel, desired, ACCEL * dt);

        let mut new_pos = peon_pos + state.vel * dt;
        new_pos.y = new_pos.y.clamp(Y_BOUND_MIN, Y_BOUND_MAX);

        if state.vel.x > 4.0 {
            state.facing_right = true;
        } else if state.vel.x < -4.0 {
            state.facing_right = false;
        }

        let frame = if state.vel.length() > WALK_ANIM_MIN_SPEED {
            state.frame_elapsed += dt;
            let frame = PEON_WALK_FRAMES[state.walk_step];
            if state.frame_elapsed >= WALK_FRAME_DURATION {
                state.frame_elapsed = 0.0;
                state.walk_step = (state.walk_step + 1) % PEON_WALK_FRAMES.len();
            }
            frame
        } else {
            PEON_WALK_FRAMES[0]
        };

        (new_pos, frame)
    }

    fn begin_death_fx(&mut self, peon: u64) {
        let (peon_ref, mut fx) = {
            let Some(peon_entity) = self.store.get_by_id::<Peon>(peon) else {
                return;
            };
            let peon_ref = peon_entity.entity_ref();

            let hit = self
                .store
                .get_child::<ImpactFrame>(&peon_entity)
                .and_then(|frame| self.store.get_child::<StaticImage>(&frame))
                .filter(|image| image.visible);

            let fx = match hit {
                Some(image) => {
                    let dest_size = image.size * image.scale;
                    DeathFx {
                        texture: image.source.clone(),
                        source: Rect::new(0.0, 0.0, image.source.width(), image.source.height()),
                        position: image.position,
                        dest_size,
                        flip_x: image.flip_x,
                        age: 0.0,
                        particles: Vec::new(),
                    }
                }
                None => {
                    let Some(body) = self.store.get_child::<Animation>(&peon_entity) else {
                        return;
                    };
                    let source = Rect::new(
                        body.current_frame as f32 * body.frame_width,
                        0.0,
                        body.frame_width,
                        body.frame_height,
                    );
                    let dest_size = if body.dest_size == Vec2::ZERO {
                        vec2(body.frame_width, body.frame_height)
                    } else {
                        body.dest_size
                    } * body.scale;
                    DeathFx {
                        texture: body.source.clone(),
                        source,
                        position: body.position,
                        dest_size,
                        flip_x: body.flip_x,
                        age: 0.0,
                        particles: Vec::new(),
                    }
                }
            };
            (peon_ref, fx)
        };
        self.store.remove(&[peon_ref]);
        self.spawn_death_particles(&mut fx);
        self.death_fx.push(fx);
    }

    fn spawn_death_particles(&self, fx: &mut DeathFx) {
        for _ in 0..DEATH_PARTICLE_COUNT {
            let r0 = gen_range(0.0, 1.0);
            let r1 = gen_range(0.0, 1.0);
            let r2 = gen_range(0.0, 1.0);
            let r3 = gen_range(0.0, 1.0);

            let pos = fx.position + vec2(r0 * fx.dest_size.x, r1 * fx.dest_size.y);
            let vel = vec2((r0 - 0.5) * 40.0, -(20.0 + r2 * 30.0));

            let color = if r3 < 0.33 {
                DUST_LIGHT
            } else if r3 < 0.66 {
                DUST_MID
            } else {
                DUST_DARK
            };

            fx.particles.push(DeathParticle {
                pos,
                vel,
                size: 2.0 + r2 * 3.0,
                delay: r1 * 0.2,
                color,
            });
        }
    }

    fn draw_death_fx(&self, fx: &DeathFx) {
        let progress = (fx.age / DEATH_DURATION).clamp(0.0, 1.0);
        let alpha = 1.0 - progress;

        if alpha > 0.01 {
            let tint = Color::new(1.0, 1.0, 1.0, alpha);
            draw_texture_ex(
                &fx.texture,
                fx.position.x,
                fx.position.y,
                tint,
                DrawTextureParams {
                    source: Some(fx.source),
                    dest_size: Some(fx.dest_size),
                    flip_x: fx.flip_x,
                    ..Default::default()
                },
            );
        }

        for p in &fx.particles {
            let local_age = (fx.age - p.delay).max(0.0);
            if local_age <= 0.0 {
                continue;
            }
            let p_progress = (local_age / DEATH_DURATION).clamp(0.0, 1.0);
            let p_alpha = (1.0 - p_progress) * alpha;
            if p_alpha <= 0.01 {
                continue;
            }

            let pos = p.pos + p.vel * local_age;
            let size = p.size * (1.0 - p_progress * 0.5);

            draw_rectangle(
                pos.x - size * 0.5,
                pos.y - size * 0.5,
                size,
                size,
                Color::new(p.color.r, p.color.g, p.color.b, p_alpha),
            );
        }
    }
}

impl System for PeonSystem {
    fn update(&mut self, ctx: &mut Context) {
        let dt = ctx.dt;

        while let Some(event) = self.squad_queue.borrow_mut().pop_front() {
            for _ in 0..event.count {
                self.spawn_peon();
            }
        }

        let death_events: Vec<PeonDeathEvent> = self.death_queue.borrow_mut().drain(..).collect();
        for event in death_events {
            self.begin_death_fx(event.peon);
        }

        for fx in &mut self.death_fx {
            fx.age += dt;
        }
        self.death_fx.retain(|fx| fx.age < DEATH_DURATION);

        let enemies: Vec<(u64, Vec2)> = self
            .store
            .all::<RamHead>()
            .filter_map(|enemy| {
                let id = enemy.entity_ref().id();
                let alive = self
                    .store
                    .get_child::<EnemyStats>(&enemy)
                    .map(|s| s.hp > 0)
                    .unwrap_or(true);
                if !alive {
                    return None;
                }
                self.store
                    .get_child::<Animation>(&enemy)
                    .map(|anim| (id, anim.position))
            })
            .collect();

        let peons: Vec<(u64, pico_entity_store::prelude::EntityRef)> = self
            .store
            .all::<Peon>()
            .filter_map(|peon| {
                let id = peon.entity_ref().id();
                let alive = self
                    .store
                    .get_child::<PeonStats>(&peon)
                    .map(|s| s.hp > 0)
                    .unwrap_or(true);
                if !alive {
                    return None;
                }
                self.store
                    .get_child::<Animation>(&peon)
                    .map(|anim| (id, anim.entity_ref()))
            })
            .collect();

        let peon_centers: Vec<(u64, Vec2)> = peons
            .iter()
            .filter_map(|(id, anim_ref)| {
                self.store
                    .get_by_id::<Animation>(anim_ref.id())
                    .map(|a| {
                        (
                            *id,
                            vec2(
                                a.position.x + PEON_FRAME_SIZE / 2.0,
                                a.position.y + PEON_FRAME_SIZE / 2.0,
                            ),
                        )
                    })
            })
            .collect();

        // Stable formation order: slot index = position in the sorted id list.
        let mut slot_order: Vec<u64> = peons.iter().map(|(id, _)| *id).collect();
        slot_order.sort_unstable();

        let knight_pos: Option<Vec2> = self
            .store
            .first::<PlayerOne>()
            .and_then(|player| self.store.get_child::<Knight>(&player))
            .and_then(|knight| self.store.get_child::<Animation>(&knight))
            .map(|anim| anim.position);
        let knight_x = knight_pos.map(|p| p.x);

        let mut anim_writes: Vec<AnimWrite> = Vec::new();
        let mut area_writes: Vec<AttackRectWrite> = Vec::new();
        let mut attack_events: Vec<AttackEvent> = Vec::new();

        for (peon_id, anim_ref) in &peons {
            let state = self.states.entry(*peon_id).or_default();

            let Some(peon_pos) = self
                .store
                .get_by_id::<Animation>(anim_ref.id())
                .map(|a| a.position)
            else {
                continue;
            };

            let nearest_enemy = enemies.iter().min_by(|a, b| {
                let dist_a = (a.1 - peon_pos).length();
                let dist_b = (b.1 - peon_pos).length();
                dist_a
                    .partial_cmp(&dist_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            let distance_to_enemy = nearest_enemy
                .map(|(_, pos)| (*pos - peon_pos).length())
                .unwrap_or(f32::MAX);

            let attacking = matches!(state.state, PeonState::AttackHit1 | PeonState::AttackHit2);

            match state.state {
                PeonState::Walk => {
                    if distance_to_enemy <= ATTACK_RANGE {
                        state.state = PeonState::AttackHit1;
                        state.state_timer = 0.0;
                        state.frame_elapsed = 0.0;
                        state.attack_step = 0;
                        state.hit_this_attack = false;
                        state.vel = Vec2::ZERO;
                        state.facing_right = nearest_enemy
                            .map(|(_, p)| p.x >= peon_pos.x)
                            .unwrap_or(true);
                    } else if peon_pos.x > self.map_w + DESPAWN_MARGIN {
                        self.store.remove(&[*anim_ref]);
                        self.states.remove(peon_id);
                        continue;
                    } else if knight_x.map(|kx| peon_pos.x > kx + MAX_PAST_KNIGHT).unwrap_or(false) {
                        state.state = PeonState::Return;
                    } else {
                        let chasing = distance_to_enemy <= CHASE_RANGE;
                        let slot_index = slot_order.binary_search(peon_id).ok();
                        let target: Option<(Vec2, f32)> = if chasing {
                            nearest_enemy.map(|(_, enemy_pos)| (*enemy_pos, ENGAGE_DIST))
                        } else {
                            slot_index
                                .zip(knight_pos)
                                .map(|(i, kpos)| (Self::formation_slot(i, kpos), 0.0))
                        };

                        let (new_pos, frame) =
                            Self::locomotion(state, *peon_id, peon_pos, target, &peon_centers, dt);

                        anim_writes.push(AnimWrite {
                            anim_ref: *anim_ref,
                            frame,
                            flip_x: !state.facing_right,
                            tint: Color::new(1.0, 1.0, 1.0, 1.0),
                            position: Some(new_pos),
                        });
                    }
                }
                PeonState::Return => {
                    let slot = slot_order
                        .binary_search(peon_id)
                        .ok()
                        .zip(knight_pos)
                        .map(|(i, kpos)| Self::formation_slot(i, kpos));

                    if distance_to_enemy <= ATTACK_RANGE {
                        state.state = PeonState::AttackHit1;
                        state.state_timer = 0.0;
                        state.frame_elapsed = 0.0;
                        state.attack_step = 0;
                        state.hit_this_attack = false;
                        state.vel = Vec2::ZERO;
                        state.facing_right = nearest_enemy
                            .map(|(_, p)| p.x >= peon_pos.x)
                            .unwrap_or(true);
                    } else if slot
                        .map(|s| (s - peon_pos).length() < SLOT_ARRIVE_RADIUS)
                        .unwrap_or(true)
                    {
                        state.state = PeonState::Walk;
                        state.walk_step = 0;
                        state.frame_elapsed = 0.0;
                    } else if let Some(slot) = slot {
                        let (new_pos, frame) = Self::locomotion(
                            state,
                            *peon_id,
                            peon_pos,
                            Some((slot, 0.0)),
                            &peon_centers,
                            dt,
                        );

                        anim_writes.push(AnimWrite {
                            anim_ref: *anim_ref,
                            frame,
                            flip_x: !state.facing_right,
                            tint: Color::new(1.0, 1.0, 1.0, 1.0),
                            position: Some(new_pos),
                        });
                    }
                }
                PeonState::AttackHit1 => {
                    state.frame_elapsed += dt;
                    let frame = PEON_ATTACK_HIT1_FRAMES[state.attack_step];
                    if state.frame_elapsed >= ATTACK_FRAME_DURATION {
                        state.frame_elapsed = 0.0;
                        state.attack_step += 1;
                        if state.attack_step >= PEON_ATTACK_HIT1_FRAMES.len() {
                            state.state = PeonState::AttackHit2;
                            state.attack_step = 0;
                        }
                    }

                    anim_writes.push(AnimWrite {
                        anim_ref: *anim_ref,
                        frame,
                        flip_x: !state.facing_right,
                        tint: Color::new(1.0, 1.0, 1.0, 1.0),
                        position: None,
                    });
                }
                PeonState::AttackHit2 => {
                    state.frame_elapsed += dt;
                    let frame = PEON_ATTACK_HIT2_FRAMES[state.attack_step];
                    if state.frame_elapsed >= ATTACK_FRAME_DURATION {
                        state.frame_elapsed = 0.0;
                        state.attack_step += 1;
                        if state.attack_step >= PEON_ATTACK_HIT2_FRAMES.len() {
                            state.state = PeonState::Recover;
                            state.state_timer = RECOVER_SECS;
                        }
                    }

                    anim_writes.push(AnimWrite {
                        anim_ref: *anim_ref,
                        frame,
                        flip_x: !state.facing_right,
                        tint: Color::new(1.0, 1.0, 1.0, 1.0),
                        position: None,
                    });
                }
                PeonState::Recover => {
                    state.state_timer -= dt;
                    let frame = PEON_ATTACK_HIT2_FRAMES[PEON_ATTACK_HIT2_FRAMES.len() - 1];

                    anim_writes.push(AnimWrite {
                        anim_ref: *anim_ref,
                        frame,
                        flip_x: !state.facing_right,
                        tint: Color::new(1.0, 1.0, 1.0, 1.0),
                        position: None,
                    });

                    if state.state_timer <= 0.0 {
                        state.state = PeonState::Walk;
                        state.walk_step = 0;
                        state.frame_elapsed = 0.0;
                    }
                }
            }

            if let Some(area_ref) = self
                .store
                .get_by_id::<Peon>(*peon_id)
                .and_then(|peon| self.store.get_child::<AttackRect>(&peon))
                .map(|area| area.entity_ref())
            {
                let body_rect = self
                    .store
                    .get_by_id::<Animation>(anim_ref.id())
                    .map(|a| a.rect())
                    .unwrap_or(Rect::new(0.0, 0.0, 0.0, 0.0));

                let rects = if attacking {
                    vec![Self::attack_rect(body_rect, state.facing_right)]
                } else {
                    Vec::new()
                };

                area_writes.push(AttackRectWrite {
                    area_ref,
                    rects: rects.clone(),
                    visible: attacking,
                });

                if attacking && !state.hit_this_attack && !rects.is_empty() {
                    let atk_rect = rects[0];
                    let mut targets = Vec::new();
                    for (enemy_id, _) in &enemies {
                        if let Some(enemy_anim) = self
                            .store
                            .get_by_id::<RamHead>(*enemy_id)
                            .and_then(|e| self.store.get_child::<Animation>(&e))
                        {
                            let er = enemy_anim.rect();
                            if atk_rect.overlaps(&er) {
                                targets.push(*enemy_id);
                            }
                        }
                    }
                    if !targets.is_empty() {
                        attack_events.push(AttackEvent {
                            attacker: *peon_id,
                            targets,
                        });
                        state.hit_this_attack = true;
                    }
                }
            }
        }

        for w in anim_writes {
            let AnimWrite {
                anim_ref,
                frame,
                flip_x,
                tint,
                position,
            } = w;
            self.store.update::<Animation, _>(&anim_ref, |animation| {
                animation.current_frame = frame;
                animation.flip_x = flip_x;
                animation.tint = tint;
                if let Some(pos) = position {
                    animation.position = pos;
                }
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

        for event in attack_events {
            self.bus.fire(&event);
        }

        while let Some(event) = self.attack_queue.borrow_mut().pop_front() {
            let is_peon = self.store.get_by_id::<Peon>(event.attacker).is_some();
            if !is_peon {
                continue;
            }

            let raw_damage = self
                .store
                .get_by_id::<Peon>(event.attacker)
                .and_then(|p| self.store.get_child::<PeonStats>(&p))
                .map(|s| s.strength)
                .unwrap_or(1)
                .max(1);

            for target in event.targets {
                let stats_ref = self
                    .store
                    .get_by_id::<RamHead>(target)
                    .and_then(|e| self.store.get_child::<EnemyStats>(&e))
                    .map(|s| s.entity_ref());
                if let Some(stats_ref) = stats_ref {
                    let armor = self
                        .store
                        .get_by_id::<EnemyStats>(stats_ref.id())
                        .map(|s| s.armor)
                        .unwrap_or(0);
                    let damage = mitigated_damage(raw_damage, armor);

                    let was_alive = self
                        .store
                        .get_by_id::<EnemyStats>(stats_ref.id())
                        .map(|s| s.hp > 0)
                        .unwrap_or(false);

                    self.store.update::<EnemyStats, _>(&stats_ref, |s| {
                        s.hp = (s.hp - damage).max(0);
                    });

                    let is_dead = self
                        .store
                        .get_by_id::<EnemyStats>(stats_ref.id())
                        .map(|s| s.hp <= 0)
                        .unwrap_or(false);

                    if was_alive && is_dead {
                        self.bus.fire(&EnemyDeathEvent { enemy: target });
                    }

                    let rect = self
                        .store
                        .get_by_id::<RamHead>(target)
                        .and_then(|e| self.store.get_child::<Animation>(&e))
                        .map(|a| a.rect());
                    if let Some(rect) = rect {
                        self.bus.fire(&HealthChangeEvent {
                            entity: target,
                            amount: -damage,
                            rect,
                        });
                    }
                }

                self.bus.fire(&HitEvent {
                    victim: target,
                    attacker: event.attacker,
                });
            }
        }

        while let Some(event) = self.enemy_attack_queue.borrow_mut().pop_front() {
            let Some((stats_ref, rect)) =
                self.store.get_by_id::<Peon>(event.target).and_then(|p| {
                    let sr = self.store.get_child::<PeonStats>(&p)?.entity_ref();
                    let rect = self.store.get_child::<Animation>(&p)?.rect();
                    Some((sr, rect))
                })
            else {
                continue;
            };

            // Consecration lets the peon deduct more armor while its feet stand
            // inside the holy ground (same buff the knight receives).
            let in_consecration = self
                .store
                .first::<Consecration>()
                .map(|area| {
                    let feet_x = rect.center().x;
                    let feet_y = rect.y + rect.h;
                    feet_x >= area.rect.x
                        && feet_x <= area.rect.x + area.rect.w
                        && feet_y >= area.rect.y
                        && feet_y <= area.rect.y + area.rect.h
                })
                .unwrap_or(false);

            let damage = self
                .store
                .get_by_id::<PeonStats>(stats_ref.id())
                .map(|stats| stats.receive_damage(event.damage, in_consecration))
                .unwrap_or(event.damage);

            let was_alive = self
                .store
                .get_by_id::<PeonStats>(stats_ref.id())
                .map(|s| s.hp > 0)
                .unwrap_or(false);

            self.store.update::<PeonStats, _>(&stats_ref, |s| {
                s.hp = (s.hp - damage).max(0);
            });

            let is_dead = self
                .store
                .get_by_id::<PeonStats>(stats_ref.id())
                .map(|s| s.hp <= 0)
                .unwrap_or(false);

            if was_alive && is_dead {
                self.bus.fire(&PeonDeathEvent { peon: event.target });
            }

            self.bus.fire(&HealthChangeEvent {
                entity: event.target,
                amount: -damage,
                rect,
            });
        }
    }

    fn draw(&self, _ctx: &Context) {
        for fx in &self.death_fx {
            self.draw_death_fx(fx);
        }
    }
}
