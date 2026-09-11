use crate::entity::dash::Dash;
use crate::entity::knight::{AttackKind, AttackPhase, Facing, Knight, IDLE_FRAME, WALK_FRAMES};
use crate::entity::knight::{MOVE_SPEED, MOVE_SPEED_VERTICAL, REVERSE_MULT, WALK_FRAME_DURATION};
use crate::entity::player::PlayerOne;
use crate::input::{self, Input};
use crate::systems::Update;
use crate::{Animation, Context, EStore};
use std::rc::Rc;

const FACE_LOCK_SECS: f32 = 0.25;

pub struct KnightControlSystem {
    store: Rc<EStore>,
    walk_elapsed: f32,
    walk_step: usize,
    phase: AttackPhase,
    phase_timer: f32,
    current_attack: Option<AttackKind>,
    buffered: Option<AttackKind>,
    prev_facing: Option<Facing>,
    combo_reset: bool,
    locked_facing: Option<Facing>,
    lock_timer: f32,
}

impl KnightControlSystem {
    pub fn new(store: Rc<EStore>) -> Self {
        Self {
            store,
            walk_elapsed: 0.0,
            walk_step: 0,
            phase: AttackPhase::Idle,
            phase_timer: 0.0,
            current_attack: None,
            buffered: None,
            prev_facing: None,
            combo_reset: false,
            locked_facing: None,
            lock_timer: 0.0,
        }
    }

    fn last_scheduled(&self) -> Option<AttackKind> {
        self.buffered.or(self.current_attack)
    }

    fn start_next_attack(&mut self) {
        if let Some(kind) = self.buffered.take() {
            self.current_attack = Some(kind);
            self.phase = AttackPhase::Windup;
            self.phase_timer = kind.windup();
        }
    }
}

impl Update for KnightControlSystem {
    fn update(&mut self, ctx: &mut Context) {
        let Some(anim_ref) = self
            .store
            .first::<PlayerOne>()
            .and_then(|p| self.store.get_child::<Knight>(&p))
            .and_then(|k| self.store.get_by_id::<Knight>(k.id()))
            .and_then(|k| self.store.get_child::<Animation>(&k))
            .map(|a| a.entity_ref())
        else {
            return;
        };

        let (pos_x, pos_y) = {
            let Some(animation) = self.store.get_by_id::<Animation>(anim_ref.id()) else {
                return;
            };
            (animation.position.x, animation.position.y)
        };

        let Some(knight_ref) = self
            .store
            .first::<PlayerOne>()
            .and_then(|p| self.store.get_child::<Knight>(&p))
            .map(|k| k.entity_ref())
        else {
            return;
        };

        let left = input::down(Input::Left);
        let right = input::down(Input::Right);
        let dir: f32 = if left && !right {
            -1.0
        } else if right && !left {
            1.0
        } else {
            0.0
        };

        if dir != 0.0 && !input::down(Input::Shift) {
            let facing = if dir < 0.0 {
                Facing::Left
            } else {
                Facing::Right
            };
            let facing_ref = self
                .store
                .get_by_id::<Knight>(knight_ref.id())
                .and_then(|k| self.store.get_child::<Facing>(&k))
                .map(|f| f.entity_ref());
            if let Some(f) = facing_ref {
                self.store.update::<Facing, _>(&f, |f| *f = facing);
            }
        }

        let current_facing = self
            .store
            .get_by_id::<Knight>(knight_ref.id())
            .and_then(|k| self.store.get_child::<Facing>(&k))
            .map(|f| *f);
        if let (Some(prev), Some(cur)) = (self.prev_facing, current_facing) {
            if prev != cur {
                self.buffered = None;
                self.combo_reset = true;
            }
        }
        self.prev_facing = current_facing;

        let up = input::down(Input::Up);
        let down = input::down(Input::Down);
        let dir_y: f32 = if up && !down {
            -1.0
        } else if down && !up {
            1.0
        } else {
            0.0
        };

        if input::down_once(Input::Attack) && self.buffered.is_none() {
            let last = if self.combo_reset {
                self.combo_reset = false;
                None
            } else {
                self.last_scheduled()
            };
            let kind = match last {
                Some(AttackKind::Thrust) => AttackKind::Swipe,
                _ => AttackKind::Thrust,
            };
            self.buffered = Some(kind);
        }

        if self.phase != AttackPhase::Idle {
            self.phase_timer -= ctx.dt;
            if self.phase_timer <= 0.0 {
                match self.phase {
                    AttackPhase::Windup => {
                        self.phase = AttackPhase::Strike;
                        self.phase_timer =
                            self.current_attack.map(AttackKind::strike).unwrap_or(0.0);
                        self.locked_facing = current_facing;
                        self.lock_timer = FACE_LOCK_SECS;
                    }
                    AttackPhase::Strike => {
                        self.phase = AttackPhase::Recovery;
                        self.phase_timer =
                            self.current_attack.map(AttackKind::recovery).unwrap_or(0.0);
                    }
                    AttackPhase::Recovery => {
                        self.phase = AttackPhase::Idle;
                        self.phase_timer = 0.0;
                        self.current_attack = None;
                    }
                    AttackPhase::Idle => {}
                }
            }
        }

        if self.phase == AttackPhase::Idle {
            self.start_next_attack();
        }

        if self.lock_timer > 0.0 {
            self.lock_timer -= ctx.dt;
            if let Some(locked) = self.locked_facing {
                if current_facing != Some(locked) {
                    let facing_ref = self
                        .store
                        .get_by_id::<Knight>(knight_ref.id())
                        .and_then(|k| self.store.get_child::<Facing>(&k))
                        .map(|f| f.entity_ref());
                    if let Some(f) = facing_ref {
                        self.store.update::<Facing, _>(&f, |f| *f = locked);
                    }
                }
            }
        } else {
            self.locked_facing = current_facing;
        }

        let mut new_pos_x = pos_x;
        let mut new_pos_y = pos_y;
        let new_frame = match self.phase {
            AttackPhase::Strike => self.current_attack.map_or(IDLE_FRAME, AttackKind::frame),
            AttackPhase::Recovery => {
                self.walk_elapsed = 0.0;
                IDLE_FRAME
            }
            _ => {
                if self
                    .store
                    .get_by_id::<Knight>(knight_ref.id())
                    .and_then(|k| self.store.get_child::<Dash>(&k))
                    .map(|d| d.time > 0.0)
                    .unwrap_or(false)
                {
                    self.walk_elapsed = 0.0;
                    IDLE_FRAME
                } else if dir != 0.0 || dir_y != 0.0 {
                    let backward = input::down(Input::Shift)
                        && current_facing.is_some_and(|f| {
                            matches!((f, dir), (Facing::Left, 1.0) | (Facing::Right, -1.0))
                        });
                    let speed = if backward { REVERSE_MULT } else { 1.0 };
                    new_pos_x += dir * MOVE_SPEED * speed * ctx.dt;
                    new_pos_y += dir_y * MOVE_SPEED_VERTICAL * ctx.dt;
                    self.walk_elapsed += ctx.dt;
                    let frame_duration = WALK_FRAME_DURATION / speed;
                    if self.walk_elapsed >= frame_duration {
                        self.walk_elapsed = 0.0;
                        let n = WALK_FRAMES.len();
                        self.walk_step = if backward {
                            (self.walk_step + n - 1) % n
                        } else {
                            (self.walk_step + 1) % n
                        };
                    }
                    WALK_FRAMES[self.walk_step]
                } else {
                    self.walk_elapsed = 0.0;
                    IDLE_FRAME
                }
            }
        };

        self.store.update::<Animation, _>(&anim_ref, |animation| {
            animation.position.x = new_pos_x.round();
            animation.position.y = new_pos_y.round();
            animation.current_frame = new_frame;
        });
    }
}
