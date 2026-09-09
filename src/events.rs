use macroquad::prelude::Rect;

#[derive(Clone, Debug)]
pub struct AttackEvent {
    pub attacker: u64,
    pub targets: Vec<u64>,
}

#[derive(Clone, Debug)]
pub struct HealthChangeEvent {
    pub entity: u64,
    pub amount: i32,
    pub rect: Rect,
}
