use std::rc::Rc;

use crate::util::{GameCamera, GameRenderTarget};
use crate::{Assets, Config, EStore, EventBus};

pub struct DiContainer {
    config: Rc<Config>,
    event_bus: Rc<EventBus>,
    estore: Rc<EStore>,
    camera: Rc<GameCamera>,
    _render_target: GameRenderTarget,
}

impl DiContainer {
    pub fn new() -> Self {
        let config = Rc::new(Config::default());
        let render_target = GameRenderTarget::new(&config);
        let camera = Rc::new(GameCamera::new(&config, &render_target));
        let event_bus = Rc::new(EventBus::new());
        let estore = Rc::new(EStore::new());

        Self {
            config,
            event_bus,
            estore,
            camera,
            _render_target: render_target,
        }
    }

    pub fn config(&self) -> Rc<Config> {
        Rc::clone(&self.config)
    }

    pub fn event_bus(&self) -> Rc<EventBus> {
        Rc::clone(&self.event_bus)
    }

    pub fn estore(&self) -> Rc<EStore> {
        Rc::clone(&self.estore)
    }

    pub fn camera(&self) -> Rc<GameCamera> {
        Rc::clone(&self.camera)
    }

    pub fn assets(&self) -> Assets {
        Assets::new()
    }
}

impl Default for DiContainer {
    fn default() -> Self {
        Self::new()
    }
}
