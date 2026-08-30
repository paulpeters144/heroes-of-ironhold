use super::sys_orb::Orb;
use crate::systems::Update;
use crate::{Context, EStore};
use macroquad::prelude::Vec2;
use std::rc::Rc;

const CAMERA_SMOOTHING: f32 = 3.0;

pub struct CameraSystem {
    store: Rc<EStore>,
    view_w: f32,
    view_h: f32,
    map_w: f32,
    map_h: f32,
    snap_next: bool,
    smooth: Vec2,
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
        }
    }

    fn locate_orb(&self) -> Option<Vec2> {
        self.store.first::<Orb>().map(|orb| orb.pos)
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

fn smooth_toward(current: Vec2, target: Vec2, dt: f32) -> Vec2 {
    let k = 1.0 - (-dt * CAMERA_SMOOTHING).exp();
    current.lerp(target, k)
}

impl Update for CameraSystem {
    fn update(&mut self, ctx: &mut Context) {
        let Some(body) = self.locate_orb() else {
            return;
        };

        let target = Vec2::new(
            self.clamp_axis(body.x, self.map_w, self.view_w),
            self.clamp_axis(body.y, self.map_h, self.view_h),
        );

        if self.snap_next {
            self.smooth = target;
            self.snap_next = false;
        } else {
            self.smooth = smooth_toward(self.smooth, target, ctx.dt);
        }

        ctx.cam_target = self.smooth.round();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 60.0;

    #[test]
    fn converges_to_target_pixel() {
        let target = Vec2::new(100.0, 100.0);
        let mut cam = Vec2::ZERO;

        for _ in 0..1000 {
            cam = smooth_toward(cam, target, DT);
        }

        assert_eq!(cam.round(), target.round());
    }

    #[test]
    fn moves_in_sub_pixel_increments() {
        let target = Vec2::new(110.0, 100.0);
        let cam = Vec2::new(100.0, 100.0);
        let next = smooth_toward(cam, target, DT);

        assert!(next.x > cam.x, "small delta must still make progress");
        assert!(next.x < target.x, "must not jump past the target");
    }

    #[test]
    fn never_overshoots_target() {
        let target = Vec2::new(200.0, 200.0);
        let mut cam = Vec2::ZERO;

        for _ in 0..1000 {
            cam = smooth_toward(cam, target, DT);
            assert!(cam.x <= target.x && cam.y <= target.y);
        }
    }
}
