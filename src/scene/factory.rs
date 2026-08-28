use super::{AssetPreviewScene, Scene};
use crate::{Assets, Config, EStore};
use di_container::Container;

#[derive(Clone, Copy)]
pub enum SceneId {
    AssetPreview,
}

#[derive(Clone, Copy)]
pub struct SceneFactory {
    cfg: &'static Config,
    container: &'static Container,
}

impl SceneFactory {
    pub fn new(cfg: &'static Config, container: &'static Container) -> Self {
        Self { cfg, container }
    }

    pub async fn create(&self, id: SceneId) -> Box<dyn Scene> {
        match id {
            SceneId::AssetPreview => {
                let assets = self
                    .container
                    .resolve_transient::<Assets>()
                    .await
                    .expect("failed to resolve Assets");
                let store = self
                    .container
                    .get::<EStore>()
                    .expect("failed to resolve EStore");
                Box::new(AssetPreviewScene::new(self.cfg, assets, store))
            }
        }
    }
}
