use di_container::{BuildContext, Injectable};
use macroquad::prelude::*;
use std::future::Future;
use std::pin::Pin;

pub struct Config {
    pub w_title: String,
    pub v_width: f32,
    pub v_height: f32,
    pub win_w: u32,
    pub win_h: u32,
    pub jitter_free: bool,
    pub rt_overscan: f32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            w_title: "Heroes of Ironhold".to_string(),
            v_width: 640.0 * 0.75,
            v_height: 360.0 * 0.75,
            win_w: 1280,
            win_h: 720,
            jitter_free: true,
            rt_overscan: 2.0,
        }
    }
}

impl Config {
    pub fn window_conf(&self) -> Conf {
        Conf {
            window_title: self.w_title.clone(),
            window_width: self.win_w as i32,
            window_height: self.win_h as i32,
            ..Default::default()
        }
    }

    pub fn rt_width(&self) -> f32 {
        self.v_width * self.rt_overscan
    }

    pub fn rt_height(&self) -> f32 {
        self.v_height * self.rt_overscan
    }
}

impl Injectable for Config {
    fn inject(
        _ctx: &BuildContext,
    ) -> Pin<Box<dyn Future<Output = di_container::Result<Self>> + '_>> {
        Box::pin(std::future::ready(Ok(Config::default())))
    }
}
