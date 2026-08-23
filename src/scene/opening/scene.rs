use crate::scene::opening::components::Animation;
use crate::scene::opening::saves::SAVE_SLOTS;
use crate::scene::opening::sys_draw::DrawSysParms;
use crate::scene::opening::sys_menu::{MenuState, MenuSys};
use crate::scene::{AnimationSys, DrawSys, Scene};
use crate::systems::SystemAgg;
use crate::{image, Assets, Config, Context, EStore, EventBus};
use macroquad::prelude::*;
use std::future::Future;
use std::pin::Pin;

pub struct OpeningScene {
    cfg: &'static Config,
    assets: &'static Assets,
    estore: &'static EStore,
    sys: &'static SystemAgg,
    bus: &'static EventBus,
}

impl OpeningScene {
    pub fn new(
        cfg: &'static Config,
        assets: &'static Assets,
        estore: &'static EStore,
        system_agg: &'static SystemAgg,
        bus: &'static EventBus,
    ) -> Self {
        Self {
            cfg,
            assets,
            estore,
            sys: system_agg,
            bus,
        }
    }
}

impl Scene for OpeningScene {
    fn load(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        self.sys.clear();

        if let Some(texture) = self.assets.texture_from_image(image::Id::MbRun) {
            self.estore.add(
                Animation {
                    texture,
                    frame: 0,
                    frame_count: 8,
                    frame_time: 0.075,
                    elapsed: 0.0,
                    pos: vec2(self.cfg.v_width * 0.5, self.cfg.v_height * 0.5 + 6.0),
                    scale: 1.0,
                },
                &[],
            );
        }

        self.estore.add(MenuState::new(1 + SAVE_SLOTS.len()), &[]);

        self.sys.add_update(AnimationSys::new(self.estore));
        self.sys.add_update(MenuSys::new(self.estore, self.bus));
        self.sys.add_draw(DrawSys::new(DrawSysParms {
            store: self.estore,
            cfg: self.cfg,
            assets: self.assets,
        }));

        Box::pin(std::future::ready(()))
    }

    fn update(&mut self, ctx: &mut Context) {
        self.sys.update(ctx);
    }

    fn draw(&self, ctx: &Context) {
        self.sys.draw(ctx);
    }

    fn dispose(&mut self) {
        self.sys.clear();
    }
}
