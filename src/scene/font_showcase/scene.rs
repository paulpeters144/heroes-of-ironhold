use crate::input::{self, Input};
use crate::scene::{ChangeSceneEvent, Scene, SceneId};
use crate::{Assets, Config, Context, EventBus, FontTag, GameFont, TextStyle};
use macroquad::prelude::*;
use std::future::Future;
use std::pin::Pin;

const SAMPLE: &str = "The quick brown fox";

struct Row {
    label: &'static str,
    gf: Option<GameFont>,
}

pub struct FontShowcase {
    bus: &'static EventBus,
    cfg: &'static Config,
    meta_font: Option<GameFont>,
    rows: Vec<Row>,
}

impl FontShowcase {
    pub fn new(bus: &'static EventBus, cfg: &'static Config, assets: &'static Assets) -> Self {
        let meta_font = assets.get_font(&TextStyle::new(FontTag::Tiny).color(GRAY));
        let rows = vec![
            Row {
                label: "Pixellari | H1 | 48px",
                gf: assets.get_font(&TextStyle::new(FontTag::H1).color(WHITE)),
            },
            Row {
                label: "Pixellari | H2 | 32px",
                gf: assets.get_font(&TextStyle::new(FontTag::H2).color(YELLOW)),
            },
            Row {
                label: "Pixellari | H3 | 16px",
                gf: assets.get_font(&TextStyle::new(FontTag::H3).color(GREEN)),
            },
            Row {
                label: "Pixellari | Body | 16px",
                gf: assets.get_font(&TextStyle::new(FontTag::Body).color(SKYBLUE)),
            },
            Row {
                label: "04b_03 | Tiny | 8px",
                gf: assets.get_font(&TextStyle::new(FontTag::Tiny).color(RED)),
            },
        ];
        Self {
            bus,
            cfg,
            meta_font,
            rows,
        }
    }

    fn draw_text_top(&self, text: &str, gf: &GameFont, x: f32, top: f32) -> f32 {
        let dims = measure_text(text, Some(gf.font), gf.size, 1.0);
        draw_text_ex(
            text,
            x,
            top + dims.offset_y,
            TextParams {
                font: Some(gf.font),
                font_size: gf.size,
                font_scale: 1.0,
                color: gf.color,
                ..Default::default()
            },
        );
        top + dims.height
    }

    fn draw_row(&self, row: &Row, x: f32, y: &mut f32) {
        if let Some(ref meta) = self.meta_font {
            *y = self.draw_text_top(row.label, meta, x, *y);
        }
        *y += 2.0;

        if let Some(ref gf) = row.gf {
            *y = self.draw_text_top(SAMPLE, gf, x, *y);
        }
        *y += 8.0;
    }
}

impl Scene for FontShowcase {
    fn load(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        Box::pin(std::future::ready(()))
    }

    fn update(&mut self, ctx: &mut Context) {
        ctx.cam_zoom = 1.0;
        ctx.cam_pan = Vec2::ZERO;

        if input::down_once(Input::Enter) {
            self.bus.fire(&ChangeSceneEvent(SceneId::Opening));
        }
    }

    fn draw(&self, _ctx: &Context) {
        draw_rectangle(
            0.0,
            0.0,
            self.cfg.v_width,
            self.cfg.v_height,
            Color::new(0.05, 0.05, 0.1, 1.0),
        );

        let x = 8.0;
        let mut y = 8.0;
        for row in &self.rows {
            self.draw_row(row, x, &mut y);
        }
    }
}
