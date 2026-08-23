use macroquad::math::Rect;

/// A static collision area extracted from a Tiled object group.
#[derive(Debug, Clone)]
pub struct CollideStatic(pub Rect);
