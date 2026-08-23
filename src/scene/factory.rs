use super::{FontShowcase, OpeningScene, Scene, SceneOne, UiShowcase};
use crate::systems::SystemAgg;
use crate::{Assets, Config, EStore, EventBus};

#[derive(Clone, Copy)]
pub enum SceneId {
    UiShowcase,
    Opening,
    FontShowcase,
    SceneOne,
}

#[derive(Clone, Copy)]
pub struct SceneFactory {
    bus: &'static EventBus,
    cfg: &'static Config,
    assets: &'static Assets,
    estore: &'static EStore,
    system_agg: &'static SystemAgg,
}

impl SceneFactory {
    pub fn new(
        bus: &'static EventBus,
        cfg: &'static Config,
        assets: &'static Assets,
        estore: &'static EStore,
        system_agg: &'static SystemAgg,
    ) -> Self {
        Self {
            bus,
            cfg,
            assets,
            estore,
            system_agg,
        }
    }

    pub fn create(&self, id: SceneId) -> Box<dyn Scene> {
        match id {
            SceneId::UiShowcase => Box::new(UiShowcase::new(self.bus, self.cfg, self.assets)),
            SceneId::Opening => Box::new(OpeningScene::new(
                self.cfg,
                self.assets,
                self.estore,
                self.system_agg,
                self.bus,
            )),
            SceneId::FontShowcase => Box::new(FontShowcase::new(self.bus, self.cfg, self.assets)),
            SceneId::SceneOne => Box::new(SceneOne::new(
                self.cfg,
                self.assets,
                self.estore,
                self.system_agg,
            )),
        }
    }
}
