use crate::scene::scene_one::components::{Anim, AnimFrame, Player, PlayerSpriteDefs};
use crate::systems::Update;
use crate::{Context, EStore};

pub struct AnimationSys {
    store: &'static EStore,
    last_anim: Anim,
}

impl AnimationSys {
    pub fn new(store: &'static EStore) -> Self {
        Self {
            store,
            last_anim: Anim::Idle,
        }
    }
}

impl Update for AnimationSys {
    fn update(&mut self, ctx: &mut Context) {
        let Some(player) = self.store.first::<Player>() else {
            return;
        };
        let cur_anim = player.anim;
        let (frame_time, frame_count, once) = {
            let Some(defs) = self.store.first::<PlayerSpriteDefs>() else {
                return;
            };
            let def = defs.for_anim(cur_anim);
            (def.frame_time, def.frame_count, def.once)
        };
        let Some(mut frame) = self.store.get_child_mut::<AnimFrame>(player) else {
            return;
        };

        if cur_anim != self.last_anim {
            frame.elapsed = 0.0;
            frame.frame = 0;
            frame.done = false;
            if cur_anim == Anim::Idle {
                frame.idle_delay = 0.2;
            }
        }
        self.last_anim = cur_anim;

        if cur_anim == Anim::Idle && frame.idle_delay > 0.0 {
            frame.idle_delay -= ctx.dt;
            frame.frame = 0;
        } else {
            frame.elapsed += ctx.dt;
            if once {
                let idx = (frame.elapsed / frame_time) as usize;
                if idx >= frame_count {
                    frame.frame = frame_count - 1;
                    frame.done = true;
                } else {
                    frame.frame = idx;
                }
            } else {
                frame.frame = (frame.elapsed / frame_time) as usize % frame_count;
            }
        }
    }
}
