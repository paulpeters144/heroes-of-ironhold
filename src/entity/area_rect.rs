use macroquad::prelude::Rect;

/// A generic debug-visible area rectangle (not an attack, not a physical
/// collider). Used to visualize zones like the guardian shield's footprint.
#[derive(Clone, Debug)]
pub struct AreaRect {
    pub rect: Rect,
}
