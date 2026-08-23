use crate::Config;
use di_container::{BuildContext, Injectable};
use macroquad::prelude::*;
use std::future::Future;
use std::pin::Pin;

pub struct GameRenderTarget {
    pub target: RenderTarget,
}

impl Injectable for GameRenderTarget {
    fn inject(
        ctx: &BuildContext,
    ) -> Pin<Box<dyn Future<Output = di_container::Result<Self>> + '_>> {
        Box::pin(async {
            let config: &'static Config = ctx.get::<&Config>()?;
            let w = config.rt_width() as u32;
            let h = config.rt_height() as u32;
            let target = render_target(w, h);
            target.texture.set_filter(FilterMode::Nearest);
            Ok(GameRenderTarget { target })
        })
    }
}

pub struct GameCamera {
    pub camera: Camera2D,
    pub render_target: RenderTarget,
}

impl Injectable for GameCamera {
    fn inject(
        ctx: &BuildContext,
    ) -> Pin<Box<dyn Future<Output = di_container::Result<Self>> + '_>> {
        Box::pin(async {
            let config: &'static Config = ctx.get::<&Config>()?;
            let rt: &'static GameRenderTarget = ctx.get::<&GameRenderTarget>()?;
            let rt_w = config.rt_width();
            let rt_h = config.rt_height();
            let display_rect = Rect::new(
                -(rt_w - config.v_width) * 0.5,
                -(rt_h - config.v_height) * 0.5,
                rt_w,
                rt_h,
            );
            let mut camera = Camera2D::from_display_rect(display_rect);
            let render_target = rt.target.clone();
            camera.render_target = Some(render_target.clone());
            Ok(GameCamera {
                camera,
                render_target,
            })
        })
    }
}
