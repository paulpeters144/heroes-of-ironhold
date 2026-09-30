use macroquad::prelude::Rect;

/// A purely logical safe zone where ram head spawning pauses. It is never
/// drawn and carries no visuals — only the footprint the wave director tests
/// the knight's position against.
#[derive(Clone, Debug)]
pub struct RestNode {
    pub rect: Rect,
}
