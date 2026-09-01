mod access;
mod di;
pub mod entity;
pub mod input;
pub mod manager;
pub mod scene;
pub mod systems;
pub mod ui;
pub mod util;
pub use di::DiContainer;
pub use event_bus::SubCollection;
pub use util::config::Config;
pub use util::estore::EStore;
pub use util::event_bus::EventBus;

pub use access::assets::Assets;
pub use access::font::{FontTag, GameFont, TextStyle};
pub use access::ids::{file, font, images, shader, sound, texture};
pub use entity::animation::Animation;
pub use entity::factory_hero::{HeroFactory, KnightCfg, KnightParts};
pub use entity::knight::{Effect, EffectKind, FrameOffsets, HeroStats, Knight, Shield, Sword};
pub use entity::static_image::StaticImage;
use manager::Manager;
pub use util::camera::GameCamera;

pub fn window_conf() -> macroquad::prelude::Conf {
    Config::default().window_conf()
}

pub struct Context {
    pub dt: f32,
    pub cam_zoom: f32,
    pub cam_target: macroquad::prelude::Vec2,
}

pub struct Game {
    mgr: Manager,
    pub ctx: Context,
}

pub async fn init() -> Game {
    use macroquad::prelude::*;

    set_pc_assets_folder("assets");

    let di = std::rc::Rc::new(DiContainer::new());

    Game {
        mgr: Manager::new(di).await,
        ctx: Context {
            dt: 0.0,
            cam_zoom: 1.0,
            cam_target: macroquad::prelude::Vec2::ZERO,
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
