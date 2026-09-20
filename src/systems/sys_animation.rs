use crate::prelude::*;
use std::rc::Rc;

pub struct AnimationUpdateSystem {
    store: Rc<EStore>,
    timers: Vec<(u64, f32)>,
}

impl AnimationUpdateSystem {
    pub fn new(store: Rc<EStore>) -> Self {
        Self {
            store,
            timers: Vec::new(),
        }
    }
}

impl System for AnimationUpdateSystem {
    fn update(&mut self, ctx: &mut Context) {
        let anims: Vec<(u64, f32, bool)> = self
            .store
            .all::<Animation>()
            .map(|a| (a.id(), a.frame_duration, a.running))
            .collect();

        for (id, frame_duration, running) in &anims {
            if !running {
                continue;
            }
            let entry = self.timers.iter_mut().find(|(tid, _)| *tid == *id);
            let elapsed = match entry {
                Some((_, e)) => {
                    *e += ctx.dt;
                    *e
                }
                None => {
                    self.timers.push((*id, ctx.dt));
                    ctx.dt
                }
            };

            if elapsed >= *frame_duration {
                if let Some((_, e)) = self.timers.iter_mut().find(|(tid, _)| *tid == *id) {
                    *e = 0.0;
                }
                if let Some(mut anim) = self.store.get_by_id_mut::<Animation>(*id) {
                    anim.current_frame = (anim.current_frame + 1) % anim.frame_count;
                }
            }
        }

        let live_ids: Vec<u64> = anims.iter().map(|(id, _, _)| *id).collect();
        self.timers.retain(|(tid, _)| live_ids.contains(tid));
    }
}