use crate::Config;
use macroquad::prelude::*;

pub struct GameRenderTarget {
    pub target: RenderTarget,
}

impl GameRenderTarget {
    pub fn new(config: &Config) -> Self {
        let w = config.rt_width() as u32;
        let h = config.rt_height() as u32;
        let target = render_target(w, h);
        target.texture.set_filter(FilterMode::Nearest);
        GameRenderTarget { target }
    }
}

pub struct GameCamera {
    pub render_target: RenderTarget,
    display_rect: Rect,
}

impl GameCamera {
    pub fn new(config: &Config, rt: &GameRenderTarget) -> Self {
        let rt_w = config.rt_width();
        let rt_h = config.rt_height();
        let display_rect = Rect::new(
            -(rt_w - config.v_width) * 0.5,
            -(rt_h - config.v_height) * 0.5,
            rt_w,
            rt_h,
        );
        let render_target = rt.target.clone();
        GameCamera {
            render_target,
            display_rect,
        }
    }

    pub fn camera_at(&self, target: Vec2) -> Camera2D {
        let mut camera = Camera2D::from_display_rect(self.display_rect);
        camera.target = target;
        camera.render_target = Some(self.render_target.clone());
        camera
    }
}
