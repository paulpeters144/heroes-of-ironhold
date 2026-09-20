use crate::input;
use crate::scene::{ChangeSceneEvent, SceneId};
use crate::systems::System;
use crate::util::view_scale;
use crate::{Assets, Config, Context, EventBus, FontTag, GameFont, TextStyle};
use macroquad::prelude::*;
use std::rc::Rc;

const OPTIONS: [&str; 4] = ["STORY", "ARCADE", "ACHIEVEMENTS", "OPTIONS"];

const GOLD: Color = Color::new(1.0, 0.84, 0.0, 1.0);
const BROWN: Color = Color::new(0.35, 0.25, 0.15, 1.0);
const INK: Color = Color::new(0.02, 0.02, 0.04, 1.0);

const SCROLL_X: f32 = -5.0;
const SCROLL_Y: f32 = 140.0;
const SCROLL_W: f32 = 210.0;
const SCROLL_H: f32 = 220.0;

const FONT_SIZE: u16 = 14;
const LINE_HEIGHT: f32 = 28.0;

pub struct MenuInputSystem {
    font: GameFont,
    selected: usize,
    v_width: f32,
    v_height: f32,
    bus: Rc<EventBus>,
}

impl MenuInputSystem {
    pub fn new(cfg: &Config, assets: &Assets, bus: Rc<EventBus>) -> Self {
        let base = assets.get_font(&TextStyle::new(FontTag::H3));
        let font = GameFont {
            size: FONT_SIZE,
            color: GOLD,
            ..base
        };
        Self {
            font,
            selected: 0,
            v_width: cfg.v_width,
            v_height: cfg.v_height,
            bus,
        }
    }

    fn scale(&self) -> f32 {
        view_scale::view_scale(self.v_width, self.v_height).0
    }

    fn dims(&self, text: &str) -> TextDimensions {
        let (font_size, font_scale) = view_scale::crisp_text_params(self.font.size, self.scale());
        measure_text(text, Some(&self.font.font), font_size, font_scale)
    }

    fn raw_text(&self, text: &str, x: f32, top: f32, color: Color) {
        let scale = self.scale();
        let (font_size, font_scale) = view_scale::crisp_text_params(self.font.size, scale);
        let dims = measure_text(text, Some(&self.font.font), font_size, font_scale);
        let baseline = view_scale::snap_to_pixel(top + dims.offset_y, scale);
        draw_text_ex(
            text,
            view_scale::snap_to_pixel(x, scale),
            baseline,
            TextParams {
                font: Some(&self.font.font),
                font_size,
                font_scale,
                color,
                ..Default::default()
            },
        );
    }

    fn text(&self, text: &str, x: f32, top: f32, color: Color) {
        for (dx, dy) in [
            (-1.0, 0.0),
            (1.0, 0.0),
            (0.0, -1.0),
            (0.0, 1.0),
            (-1.0, -1.0),
            (1.0, -1.0),
            (-1.0, 1.0),
            (1.0, 1.0),
        ] {
            self.raw_text(text, x + dx, top + dy, INK);
        }
        self.raw_text(text, x, top, color);
    }

    fn center_x(&self, text: &str) -> f32 {
        SCROLL_X + SCROLL_W * 0.5 - self.dims(text).width * 0.5
    }
}

impl System for MenuInputSystem {
    fn update(&mut self, _ctx: &mut Context) {
        if input::down_once(input::Input::Up) {
            self.selected = if self.selected == 0 {
                OPTIONS.len() - 1
            } else {
                self.selected - 1
            };
        }
        if input::down_once(input::Input::Down) {
            self.selected = (self.selected + 1) % OPTIONS.len();
        }
        if input::down_once(input::Input::Enter) && OPTIONS[self.selected] == "STORY" {
            self.bus.fire(&ChangeSceneEvent(SceneId::BattleTest));
        }
    }

    fn draw_ui(&self, _ctx: &Context) {
        let total_h = LINE_HEIGHT * OPTIONS.len() as f32;
        let start_y = SCROLL_Y + (SCROLL_H - total_h) * 0.5;

        for (i, option) in OPTIONS.iter().enumerate() {
            let x = self.center_x(option);
            let y = start_y + i as f32 * LINE_HEIGHT;
            let color = if i == self.selected { GOLD } else { BROWN };
            self.text(option, x, y, color);
        }
    }
}
