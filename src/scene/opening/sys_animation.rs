use crate::scene::opening::components::Animation;
use crate::systems::Update;
use crate::{Context, EStore};

pub struct AnimationSys {
    store: &'static EStore,
}

impl AnimationSys {
    pub fn new(store: &'static EStore) -> Self {
        Self { store }
    }
}

impl Update for AnimationSys {
    fn update(&mut self, ctx: &mut Context) {
        let dt = ctx.dt;
        for anim in self.store.all_mut::<Animation>() {
            anim.elapsed += dt;
            anim.frame = (anim.elapsed / anim.frame_time) as usize % anim.frame_count;
        }
    }
}
