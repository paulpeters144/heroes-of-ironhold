use macroquad::prelude::Rect;

/// Shared attack-zone component used by both the knight's melee and enemies
/// (e.g. the Ram Head's charge). `rects` holds the active hit rectangles and
/// `visible` is true while an attack is in flight.
#[derive(Clone, Debug)]
pub struct AttackRect {
    pub rects: Vec<Rect>,
    pub visible: bool,
}
