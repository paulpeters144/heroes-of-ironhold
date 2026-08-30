use crate::entity::knight::{Facing, Knight, SWIPE_FRAME, THRUST_FRAME};
use crate::systems::Update;
use crate::{Animation, Context, EStore};
use std::rc::Rc;

const FACE_LOCK_SECS: f32 = 0.25;

pub struct KnightFacingLockSystem {
    store: Rc<EStore>,
    locked_facing: Option<Facing>,
    lock_timer: f32,
    prev_frame: Option<usize>,
}

impl KnightFacingLockSystem {
    pub fn new(store: Rc<EStore>) -> Self {
        Self {
            store,
            locked_facing: None,
            lock_timer: 0.0,
            prev_frame: None,
        }
    }
}

impl Update for KnightFacingLockSystem {
    fn update(&mut self, ctx: &mut Context) {
        let (frame, facing_ref, current) = {
            let Some(knight) = self.store.first::<Knight>() else {
                return;
            };
            let frame = self
                .store
                .get_child::<Animation>(&knight)
                .map(|a| a.current_frame);
            let facing_ref = self
                .store
                .get_child::<Facing>(&knight)
                .map(|f| f.entity_ref());
            let current = match &facing_ref {
                Some(r) => self.store.get_by_id::<Facing>(r.id()).map(|f| *f),
                None => None,
            };
            (frame, facing_ref, current)
        };

        let Some(facing_ref) = facing_ref else {
            return;
        };
        let Some(current) = current else {
            return;
        };

        if let Some(frame) = frame {
            let attacked = if self.prev_frame == Some(frame) {
                false
            } else {
                self.prev_frame = Some(frame);
                matches!(frame, THRUST_FRAME | SWIPE_FRAME)
            };
            if attacked {
                self.locked_facing = Some(current);
                self.lock_timer = FACE_LOCK_SECS;
            }
        }

        if self.lock_timer > 0.0 {
            self.lock_timer -= ctx.dt;
            if let Some(locked) = self.locked_facing {
                if current != locked {
                    self.store.update::<Facing, _>(&facing_ref, |f| *f = locked);
                }
            }
        } else {
            self.locked_facing = Some(current);
        }
    }
}
