use std::collections::HashMap;

use crate::scene::scene_one::components::{AnimFrame, Enemy, EnemyAnim, EnemySpriteDefs};
use crate::systems::Update;
use crate::{Context, EStore};

pub struct EnemyAnimationSys {
    store: &'static EStore,
    last_anim: HashMap<u64, EnemyAnim>,
}

impl EnemyAnimationSys {
    pub fn new(store: &'static EStore) -> Self {
        Self {
            store,
            last_anim: HashMap::new(),
        }
    }
}

impl Update for EnemyAnimationSys {
    fn update(&mut self, ctx: &mut Context) {
        let dt = ctx.dt;
        let ids = self.store.ids::<Enemy>();
        for id in ids {
            let Some(enemy) = self.store.get_by_id::<Enemy>(id) else {
                continue;
            };
            let cur = enemy.anim;
            let (frame_time, frame_count, once) = {
                let Some(defs) = self.store.first::<EnemySpriteDefs>() else {
                    return;
                };
                let def = defs.for_anim(cur);
                (def.frame_time, def.frame_count, def.once)
            };
            let Some(mut frame) = self.store.get_child_mut::<AnimFrame>(enemy) else {
                continue;
            };

            if self.last_anim.get(&id).copied() != Some(cur) {
                frame.frame = 0;
                frame.elapsed = 0.0;
                frame.done = false;
            }
            self.last_anim.insert(id, cur);

            frame.elapsed += dt;
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
