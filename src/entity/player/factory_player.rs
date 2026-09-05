use super::{PlayerOne, PlayerTwo};

pub struct PlayerFactory;

impl PlayerFactory {
    pub fn spawn_one() -> PlayerOne {
        PlayerOne
    }

    pub fn spawn_two() -> PlayerTwo {
        PlayerTwo
    }
}
