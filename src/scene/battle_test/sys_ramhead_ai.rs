use crate::entity::enemy::{EnemyStats, RamHead, ATTACK_FRAMES, IDLE_FRAME, WALK_FRAMES};
use crate::entity::knight::{AttackArea, Knight};
use crate::entity::player::PlayerOne;
use crate::events::EnemyAttackEvent;
use crate::systems::Update;
use crate::util::attack::{did_attack, image_data_for};
use crate::{Animation, Context, EStore, EventBus};
use macroquad::prelude::{vec2, Color, Image, Rect, Texture2D, Vec2};
use std::rc::Rc;

// Chase movement: walk speed and walk frame timing.
const MOVE_SPEED: f32 = 62.5;
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Chase,
    Windup,
    Charge,
    Recover,
}

pub struct RamHeadAiSystem {
    store: Rc<EStore>,
    bus: Rc<EventBus>,
    state: State,
    state_timer: f32,
    cooldown: f32,
    frame_elapsed: f32,
    walk_step: usize,
    attack_step: usize,
    charge_dir: Vec2,
    hit_this_charge: bool,
    sheet_cache: Option<(Texture2D, Image)>,
}

impl RamHeadAiSystem {
    pub fn new(store: Rc<EStore>, bus: Rc<EventBus>) -> Self {
        Self {
            store,
            bus,
            state: State::Chase,
            state_timer: 0.0,
            cooldown: 0.0,
            frame_elapsed: 0.0,
            walk_step: 0,
            attack_step: 0,
            charge_dir: Vec2::ZERO,
            hit_this_charge: false,
            sheet_cache: None,
        }
    }
}

impl Update for RamHeadAiSystem {
    fn update(&mut self, ctx: &mut Context) {
        let Some(anim_ref) = self
            .store
            .first::<RamHead>()
            .and_then(|enemy| self.store.get_child::<Animation>(&enemy))
            .map(|a| a.entity_ref())
        else {
            return;
        };

        let Some(enemy_pos) = self
            .store
            .get_by_id::<Animation>(anim_ref.id())
            .map(|a| a.position)
        else {
            return;
        };

        let Some(knight_pos) = self
            .store
            .first::<PlayerOne>()
            .and_then(|p| self.store.get_child::<Knight>(&p))
            .and_then(|k| self.store.get_child::<Animation>(&k))
            .map(|a| a.position)
        else {
            self.store.update::<Animation, _>(&anim_ref, |animation| {
                animation.current_frame = IDLE_FRAME;
            });
            return;
        };

        let delta = knight_pos - enemy_pos;
        let distance = delta.length();

        let enraged = self
            .store
            .first::<RamHead>()
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

        self.cooldown = (self.cooldown - ctx.dt).max(0.0);

        match self.state {
            State::Chase => {
                if distance > 0.0 && self.cooldown <= 0.0 && distance <= WINDUP_RANGE {
                    self.state = State::Windup;
                    self.state_timer = windup_secs;
                    self.charge_dir = delta / distance;
                    self.frame_elapsed = 0.0;
                    self.attack_step = 0;
                }
            }
            State::Windup => {
                self.state_timer -= ctx.dt;
                if self.state_timer <= 0.0 {
                    self.state = State::Charge;
                    self.state_timer = CHARGE_SECS;
                    self.hit_this_charge = false;
                    self.frame_elapsed = 0.0;
                    self.attack_step = 0;
                }
            }
            State::Charge => {
                self.state_timer -= ctx.dt;
                if self.state_timer <= 0.0 {
                    self.state = State::Recover;
                    self.state_timer = RECOVER_SECS;
                }
            }
            State::Recover => {
                self.state_timer -= ctx.dt;
                if self.state_timer <= 0.0 {
                    self.state = State::Chase;
                    self.cooldown = cooldown_secs;
                }
            }
        }

        let mut new_pos = enemy_pos;
        let (frame, flip_x, tint) = match self.state {
            State::Chase => {
                if distance > 0.0 {
                    let dir = delta / distance;
                    new_pos += vec2(dir.x * MOVE_SPEED, dir.y * MOVE_SPEED_VERTICAL) * ctx.dt;
                }
                self.frame_elapsed += ctx.dt;
                let frame = WALK_FRAMES[self.walk_step];
                if self.frame_elapsed >= FRAME_DURATION {
                    self.frame_elapsed = 0.0;
                    self.walk_step = (self.walk_step + 1) % WALK_FRAMES.len();
                }
                (frame, delta.x < 0.0, Color::new(1.0, 1.0, 1.0, 1.0))
            }
            State::Windup => {
                new_pos += self.charge_dir * WINDUP_ADVANCE * ctx.dt;
                (
                    ATTACK_FRAMES[0],
                    self.charge_dir.x < 0.0,
                    Color::new(1.0, 0.5, 0.5, 1.0),
                )
            }
            State::Charge => {
                new_pos += self.charge_dir * charge_speed * ctx.dt;
                self.frame_elapsed += ctx.dt;
                let frame = ATTACK_FRAMES[self.attack_step];
                if self.frame_elapsed >= CHARGE_FRAME_DURATION {
                    self.frame_elapsed = 0.0;
                    self.attack_step = (self.attack_step + 1) % ATTACK_FRAMES.len();
                }
                (frame, self.charge_dir.x < 0.0, Color::new(1.0, 1.0, 1.0, 1.0))
            }
            State::Recover => (
                ATTACK_FRAMES[ATTACK_FRAMES.len() - 1],
                self.charge_dir.x < 0.0,
                Color::new(1.0, 1.0, 1.0, 1.0),
            ),
        };

        self.store.update::<Animation, _>(&anim_ref, |animation| {
            animation.position.x = new_pos.x.round();
            animation.position.y = new_pos.y.round();
            animation.current_frame = frame;
            animation.flip_x = flip_x;
            animation.tint = tint;
        });

        let Some(area_ref) = self
            .store
            .first::<RamHead>()
            .and_then(|enemy| self.store.get_child::<AttackArea>(&enemy))
            .map(|area| area.entity_ref())
        else {
            return;
        };

        let Some(body_rect) = self
            .store
            .get_by_id::<Animation>(anim_ref.id())
            .map(|anim| anim.rect())
        else {
            return;
        };

        let attack_rects: Vec<Rect> = if self.state == State::Charge {
            let mut r = body_rect;
            let dir = self.charge_dir;
            if dir.x < 0.0 {
                r.x -= CHARGE_REACH;
            }
            r.w += CHARGE_REACH * dir.x.abs();
            if dir.y < 0.0 {
                r.y -= CHARGE_REACH;
            }
            r.h += CHARGE_REACH * dir.y.abs();
            vec![r]
        } else {
            Vec::new()
        };
        let attacking = self.state == State::Charge;

        self.store.update::<AttackArea, _>(&area_ref, |area| {
            area.rects = attack_rects.clone();
            area.visible = attacking;
        });

        // A charge deals its damage once, on first contact with the knight.
        if attacking && !self.hit_this_charge {
            let data = self
                .store
                .first::<PlayerOne>()
                .and_then(|player| self.store.get_child::<Knight>(&player))
                .and_then(|knight| self.store.get_child::<Animation>(&knight))
                .map(|anim| image_data_for(&anim, &mut self.sheet_cache));

            if let Some(data) = data {
                if attack_rects.iter().any(|rect| did_attack(*rect, &data)) {
                    let attacker = self
                        .store
                        .first::<RamHead>()
                        .map(|enemy| enemy.entity_ref().id());
                    let target = self
                        .store
                        .first::<PlayerOne>()
                        .and_then(|player| self.store.get_child::<Knight>(&player))
                        .map(|knight| knight.entity_ref().id());

                    if let (Some(attacker), Some(target)) = (attacker, target) {
                        self.bus.fire(&EnemyAttackEvent { attacker, target });
                    }
                    self.hit_this_charge = true;
                }
            }
        }
    }
}
