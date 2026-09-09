#[derive(Clone, Debug)]
pub struct AttackEvent {
    pub attacker: u64,
    pub targets: Vec<u64>,
}
