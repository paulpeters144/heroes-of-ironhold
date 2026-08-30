use super::{AssetPreviewScene, BattleTestScene, Scene};
use crate::DiContainer;
use std::rc::Rc;

#[derive(Clone, Copy)]
pub enum SceneId {
    AssetPreview,
    BattleTest,
}

#[derive(Clone)]
pub struct SceneFactory {
    di: Rc<DiContainer>,
}

impl SceneFactory {
    pub fn new(di: Rc<DiContainer>) -> Self {
        Self { di }
    }

    pub fn create(&self, id: SceneId) -> Box<dyn Scene> {
        let cfg = self.di.config();
        let assets = self.di.assets();
        let store = self.di.estore();
        match id {
            SceneId::AssetPreview => Box::new(AssetPreviewScene::new(cfg, assets, store)),
            SceneId::BattleTest => Box::new(BattleTestScene::new(cfg, assets, store)),
        }
    }
}
