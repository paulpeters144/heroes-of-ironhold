use macroquad::prelude::*;

pub struct Config {
    pub w_title: String,
    pub v_width: f32,
    pub v_height: f32,
    pub win_w: u32,
    pub win_h: u32,
    pub rt_overscan: f32,
    pub pixel_snap: bool,
    pub debug: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            w_title: "Heroes of Ironhold".to_string(),
            v_width: 640.0,
            v_height: 360.0,
            win_w: 1280,
            win_h: 720,
            rt_overscan: 2.0,
            pixel_snap: true,
            debug: false,
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
