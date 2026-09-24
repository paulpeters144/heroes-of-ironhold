use macroquad::prelude::Rect;

/// The sliding window where peons spawn, behind the current wave gate.
/// Written by the wave director, read by the peon system and the camera clamp.
#[derive(Clone, Debug)]
pub struct PeonSpawnZone {
    pub rect: Rect, // world-space spawn area for peons
}

/// The sliding window where ram heads spawn, ahead of the current wave gate.
/// Written by the wave director, read by the ram head spawner.
#[derive(Clone, Debug)]
pub struct RamHeadSpawnZone {
    pub rect: Rect, // world-space spawn area for ram heads
}
