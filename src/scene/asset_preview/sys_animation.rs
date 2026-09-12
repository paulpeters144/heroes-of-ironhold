use crate::systems::System;
use crate::{Animation, Context, EStore};
use std::rc::Rc;

const FRAME_DURATION: f32 = 0.12;

pub struct AnimationUpdateSystem {
    store: Rc<EStore>,
    elapsed: f32,
}

impl AnimationUpdateSystem {
    pub fn new(store: Rc<EStore>) -> Self {
        Self {
            store,
            elapsed: 0.0,
        }
    }
}

impl System for AnimationUpdateSystem {
    fn update(&mut self, ctx: &mut Context) {
        self.elapsed += ctx.dt;
        if self.elapsed < FRAME_DURATION {
            return;
        }
        self.elapsed = 0.0;

        for mut animation in self.store.all_mut::<Animation>() {
            if !animation.running {
                continue;
            }
            animation.current_frame = (animation.current_frame + 1) % animation.frame_count;
        }
    }
}
