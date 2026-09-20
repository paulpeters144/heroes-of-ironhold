use macroquad::prelude::*;

use heroes_of_ironhold_core::window_conf;

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = heroes_of_ironhold_core::init().await;
    loop {
        heroes_of_ironhold_core::input::poll_input();
        heroes_of_ironhold_core::update(&mut game);
        heroes_of_ironhold_core::draw(&game);
        next_frame().await;
    }
}