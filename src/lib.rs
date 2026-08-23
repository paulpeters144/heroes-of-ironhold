mod access;
pub mod input;
pub mod manager;
pub mod scene;
pub mod systems;
pub mod ui;
pub mod util;
pub use event_bus::SubCollection;
pub use util::config::Config;
pub use util::estore::EStore;
pub use util::event_bus::EventBus;

pub use access::assets::Assets;
pub use access::font::{FontTag, GameFont, TextStyle};
pub use access::ids::{file, font, image, shader, sound, texture};
use di_container::ContainerBuilder;
use manager::Manager;
use systems::SystemAgg;
pub use util::camera::GameCamera;

pub fn window_conf() -> macroquad::prelude::Conf {
    Config::default().window_conf()
}

pub struct Context {
    pub dt: f32,
    pub cam_zoom: f32,
    pub cam_pan: macroquad::prelude::Vec2,
}

pub struct Game {
    mgr: Manager,
    pub ctx: Context,
}

pub async fn init() -> Game {
    use macroquad::prelude::*;
    use util::camera::GameRenderTarget;

    set_pc_assets_folder("assets");

    let container = ContainerBuilder::new()
        .singleton::<Config, Config>()
        .singleton::<EventBus, EventBus>()
        .singleton::<Assets, Assets>()
        .singleton::<EStore, EStore>()
        .singleton::<SystemAgg, SystemAgg>()
        .singleton::<GameRenderTarget, GameRenderTarget>()
        .singleton::<GameCamera, GameCamera>()
        .build_and_leak()
        .await
        .unwrap();

    Game {
        mgr: Manager::new(container),
        ctx: Context {
            dt: 0.0,
            cam_zoom: 1.0,
            cam_pan: macroquad::prelude::Vec2::ZERO,
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
