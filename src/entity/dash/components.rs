use macroquad::prelude::Vec2;

/// Static, per-hero dash stats. Held inside `Dash` so one system can drive
/// every hero class with the class's own feel.
#[derive(Clone, Copy, Debug)]
pub struct DashCfg {
    /// Movement speed during a dash, in pixels per second.
    pub speed: f32,
    /// How long a single dash lasts, in seconds.
    pub duration: f32,
    /// Seconds to refill one charge.
    pub recover_secs: f32,
    /// Total charge capacity.
    pub max_charges: u32,
}

/// Dash state + stats; attached to a hero as a child entity. Copy so systems
/// can snapshot it. Check `time > 0.0` for an active dash, not component
/// presence.
#[derive(Clone, Copy, Debug)]
pub struct Dash {
    pub cfg: DashCfg,
    pub dir: Vec2,
    /// Active-dash countdown; `> 0.0` while dashing.
    pub time: f32,
    /// Current charge count.
    pub charges: u32,
    /// Countdown until the next charge refill.
    pub recovery: f32,
}

impl Dash {
    pub fn knight() -> Self {
        Self {
            cfg: DashCfg {
                speed: 450.0,
                duration: 0.15,
                recover_secs: 3.0,
                max_charges: 3,
            },
            dir: Vec2::ZERO,
            time: 0.0,
            charges: 3,
            recovery: 0.0,
        }
    }

    pub fn assassin() -> Self {
        Self {
            cfg: DashCfg {
                speed: 600.0,
                duration: 0.10,
                recover_secs: 2.0,
                max_charges: 4,
            },
            dir: Vec2::ZERO,
            time: 0.0,
            charges: 4,
            recovery: 0.0,
        }
    }

    pub fn wizard() -> Self {
        Self {
            cfg: DashCfg {
                speed: 900.0,
                duration: 0.08,
                recover_secs: 5.0,
                max_charges: 2,
            },
            dir: Vec2::ZERO,
            time: 0.0,
            charges: 2,
            recovery: 0.0,
        }
    }
}
