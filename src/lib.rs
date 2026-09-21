mod access;
mod di;
pub mod entity;
pub mod events;
pub mod input;
pub mod manager;
pub mod prelude;
pub mod scene;
pub mod systems;
pub mod ui;
mod util;

// Infrastructure re-exports only
pub use access::ids::{file, font, images, shader, sound, texture};
pub use access::Assets;
pub use access::{DiskJsonStore, GameState, GameStateStore, SaveError};
pub use di::DiContainer;
use manager::Manager;
pub use util::Config;
pub use util::EStore;
pub use util::EventBus;

pub fn window_conf() -> macroquad::prelude::Conf {
    Config::default().window_conf()
}

pub struct Context {
    pub dt: f32,
    pub cam_zoom: f32,
    pub cam_target: macroquad::prelude::Vec2,
    pub debug: bool,
}

pub struct Game {
    mgr: Manager,
    pub ctx: Context,
}

pub async fn init() -> Game {
    use macroquad::prelude::*;

    set_pc_assets_folder("assets");

    let di = std::rc::Rc::new(DiContainer::new());
    let debug = di.config().debug;

    Game {
        mgr: Manager::new(di).await,
        ctx: Context {
            dt: 0.0,
            cam_zoom: 1.0,
            cam_target: macroquad::prelude::Vec2::ZERO,
            debug,
        },
    }
}

pub fn update(game: &mut Game) {
    game.mgr.update(&mut game.ctx);
    input::end_frame(game.ctx.dt);
}

pub fn draw(game: &Game) {
    game.mgr.draw(&game.ctx);
}
