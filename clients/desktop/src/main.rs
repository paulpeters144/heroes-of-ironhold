use macroquad::prelude::*;

use heroes_of_ironhold_core::input::Input;
use heroes_of_ironhold_core::window_conf;

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = heroes_of_ironhold_core::init().await;
    loop {
        heroes_of_ironhold_core::input::set(Input::Enter, is_key_down(KeyCode::Enter));
        heroes_of_ironhold_core::input::set(Input::Up, is_key_down(KeyCode::Up));
        heroes_of_ironhold_core::input::set(Input::Down, is_key_down(KeyCode::Down));
        heroes_of_ironhold_core::input::set(Input::Left, is_key_down(KeyCode::Left));
        heroes_of_ironhold_core::input::set(Input::Right, is_key_down(KeyCode::Right));
        heroes_of_ironhold_core::input::set(Input::Jump, is_key_down(KeyCode::Space));
        heroes_of_ironhold_core::input::set(Input::Attack, is_key_down(KeyCode::X));
        heroes_of_ironhold_core::input::set(
            Input::Shift,
            is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift),
        );
        heroes_of_ironhold_core::input::set(Input::Skills, is_key_down(KeyCode::A));
        heroes_of_ironhold_core::update(&mut game);
        heroes_of_ironhold_core::draw(&game);
        next_frame().await;
    }
}
