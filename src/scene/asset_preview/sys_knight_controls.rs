use crate::entity::knight::{Knight, IDLE_FRAME, SWIPE_FRAME, THRUST_FRAME, WALK_FRAMES};
use crate::input::{self, Input};
use crate::systems::Update;
use crate::{Animation, Context, EStore};
use pico_entity_store::entity_ref::EntityRef;

const MOVE_SPEED: f32 = 150.0;
const MOVE_SPEED_VERTICAL: f32 = MOVE_SPEED * 0.75;
const WALK_FRAME_DURATION: f32 = 0.14;

#[derive(Clone, Copy, PartialEq)]
enum AttackKind {
    Thrust,
    Swipe,
}

impl AttackKind {
    fn frame(self) -> usize {
        match self {
            AttackKind::Thrust => THRUST_FRAME,
            AttackKind::Swipe => SWIPE_FRAME,
        }
    }

    fn windup(self) -> f32 {
        match self {
            AttackKind::Thrust => 0.01,
            AttackKind::Swipe => 0.01,
        }
    }

    fn strike(self) -> f32 {
        match self {
            AttackKind::Thrust => 0.2,
            AttackKind::Swipe => 0.2,
        }
    }

    fn recovery(self) -> f32 {
        0.1
    }
}

#[derive(Clone, Copy, PartialEq)]
enum AttackPhase {
    Idle,
    Windup,
    Strike,
    Recovery,
}

pub struct KnightControlSystem {
    store: &'static EStore,
    walk_elapsed: f32,
    walk_step: usize,
    phase: AttackPhase,
    phase_timer: f32,
    current_attack: Option<AttackKind>,
    buffered: Option<AttackKind>,
}

impl KnightControlSystem {
    pub fn new(store: &'static EStore) -> Self {
        Self {
            store,
            walk_elapsed: 0.0,
            walk_step: 0,
            phase: AttackPhase::Idle,
            phase_timer: 0.0,
            current_attack: None,
            buffered: None,
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

    fn locate(&self) -> Option<EntityRef> {
        for knight in self.store.all::<Knight>() {
            if let Some(animation) = self.store.get_child::<Animation>(&knight) {
                return Some(animation.entity_ref());
            }
        }
        None
    }
}

impl Update for KnightControlSystem {
    fn update(&mut self, ctx: &mut Context) {
        let Some(anim_ref) = self.locate() else {
            return;
        };

        let (pos_x, pos_y) = {
            let Some(animation) = self.store.get_by_id::<Animation>(anim_ref.id()) else {
                return;
            };
            (animation.position.x, animation.position.y)
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
            let kind = match self.last_scheduled() {
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

        let mut new_pos_x = pos_x;
        let mut new_pos_y = pos_y;
        let new_frame = match self.phase {
            AttackPhase::Strike => self.current_attack.map_or(IDLE_FRAME, AttackKind::frame),
            AttackPhase::Recovery => {
                self.walk_elapsed = 0.0;
                IDLE_FRAME
            }
            _ => {
                if dir != 0.0 || dir_y != 0.0 {
                    new_pos_x += dir * MOVE_SPEED * ctx.dt;
                    new_pos_y += dir_y * MOVE_SPEED_VERTICAL * ctx.dt;
                    self.walk_elapsed += ctx.dt;
                    if self.walk_elapsed >= WALK_FRAME_DURATION {
                        self.walk_elapsed = 0.0;
                        if dir < 0.0 {
                            self.walk_step =
                                (self.walk_step + WALK_FRAMES.len() - 1) % WALK_FRAMES.len();
                        } else {
                            self.walk_step = (self.walk_step + 1) % WALK_FRAMES.len();
                        }
                    }
                    WALK_FRAMES[self.walk_step]
                } else {
                    self.walk_elapsed = 0.0;
                    IDLE_FRAME
                }
            }
        };

        self.store.update::<Animation, _>(&anim_ref, |animation| {
            animation.position.x = new_pos_x;
            animation.position.y = new_pos_y;
            animation.current_frame = new_frame;
        });
    }
}
