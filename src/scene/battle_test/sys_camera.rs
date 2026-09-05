use super::sys_orb::Orb;
use crate::systems::Update;
use crate::{Context, EStore};
use macroquad::prelude::Vec2;
use std::rc::Rc;

const CAMERA_FREQUENCY: f32 = 5.0;

pub struct CameraSystem {
    store: Rc<EStore>,
    view_w: f32,
    view_h: f32,
    map_w: f32,
    map_h: f32,
    snap_next: bool,
    smooth: Vec2,
    velocity: Vec2,
}

impl CameraSystem {
    pub fn new(store: Rc<EStore>, view_w: f32, view_h: f32, map_w: f32, map_h: f32) -> Self {
        Self {
            store,
            view_w,
            view_h,
            map_w,
            map_h,
            snap_next: true,
            smooth: Vec2::ZERO,
            velocity: Vec2::ZERO,
        }
    }

    fn clamp_axis(&self, value: f32, map_size: f32, view_size: f32) -> f32 {
        let half = view_size * 0.5;
        if map_size >= view_size {
            value.clamp(half, map_size - half)
        } else {
            map_size * 0.5
        }
    }
}

/// Critically damped spring toward `target`. Closed-form, so it is exact and
/// stable for any `dt` and never overshoots.
fn spring_toward(pos: Vec2, vel: Vec2, target: Vec2, dt: f32) -> (Vec2, Vec2) {
    let w = CAMERA_FREQUENCY;
    let y = pos - target;
    let a = vel + w * y;
    let e = (-w * dt).exp();
    (target + (y + a * dt) * e, (vel - w * a * dt) * e)
}

/// One camera step with a square stop zone. If the camera center is already
/// inside the zone (`zone_half` on each axis) it freezes in place with zero
/// velocity; otherwise it springs toward `target`.
fn step_camera(pos: Vec2, vel: Vec2, target: Vec2, zone_half: f32, dt: f32) -> (Vec2, Vec2) {
    let delta = target - pos;
    if delta.x.abs() <= zone_half && delta.y.abs() <= zone_half {
        (pos, Vec2::ZERO)
    } else {
        spring_toward(pos, vel, target, dt)
    }
}

impl Update for CameraSystem {
    fn update(&mut self, ctx: &mut Context) {
        let Some(orb) = self.store.first::<Orb>().map(|orb| (*orb).clone()) else {
            return;
        };

        let target = Vec2::new(
            self.clamp_axis(orb.pos.x, self.map_w, self.view_w),
            self.clamp_axis(orb.pos.y, self.map_h, self.view_h),
        );

        let zone_half = orb.size * 0.5;

        if self.snap_next {
            self.smooth = target;
            self.velocity = Vec2::ZERO;
            self.snap_next = false;
        } else {
            let (pos, vel) = step_camera(self.smooth, self.velocity, target, zone_half, ctx.dt);
            self.smooth = pos;
            self.velocity = vel;
        }

        ctx.cam_target = self.smooth;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 60.0;

    #[test]
    fn converges_to_target_pixel() {
        let target = Vec2::new(100.0, 100.0);
        let mut pos = Vec2::ZERO;
        let mut vel = Vec2::ZERO;

        for _ in 0..1000 {
            let (p, v) = spring_toward(pos, vel, target, DT);
            pos = p;
            vel = v;
        }

        assert_eq!(pos.round(), target.round());
    }

    #[test]
    fn moves_in_sub_pixel_increments() {
        let target = Vec2::new(110.0, 100.0);
        let pos = Vec2::new(100.0, 100.0);
        let (next, _) = spring_toward(pos, Vec2::ZERO, target, DT);

        assert!(next.x > pos.x, "small delta must still make progress");
        assert!(next.x < target.x, "must not jump past the target");
    }

    #[test]
    fn never_overshoots_target() {
        let target = Vec2::new(200.0, 200.0);
        let mut pos = Vec2::ZERO;
        let mut vel = Vec2::ZERO;

        for _ in 0..1000 {
            let (p, v) = spring_toward(pos, vel, target, DT);
            pos = p;
            vel = v;
            assert!(pos.x <= target.x && pos.y <= target.y);
        }
    }

    #[test]
    fn freezes_once_inside_stop_zone() {
        let target = Vec2::new(100.0, 100.0);
        let pos = Vec2::new(99.0, 100.0);
        let (next, vel) = step_camera(pos, Vec2::ZERO, target, 2.0, DT);

        assert_eq!(next, pos, "camera must not move inside the stop zone");
        assert_eq!(vel, Vec2::ZERO, "camera must have no velocity inside the zone");
    }

    #[test]
    fn springs_when_outside_stop_zone() {
        let target = Vec2::new(100.0, 100.0);
        let pos = Vec2::new(0.0, 0.0);
        let (next, _) = step_camera(pos, Vec2::ZERO, target, 2.0, DT);

        assert!(next.x > pos.x, "camera must move toward the target outside the zone");
    }
}
