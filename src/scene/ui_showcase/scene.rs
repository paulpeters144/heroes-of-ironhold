use crate::input::{self, Input};
use crate::scene::{ChangeSceneEvent, Scene, SceneId};
use crate::ui::{Style, UI};
use crate::{Assets, Config, Context, EventBus, FontTag, GameFont, TextStyle};
use macroquad::prelude::*;
use std::future::Future;
use std::pin::Pin;

pub struct UiShowcase {
    bus: &'static EventBus,
    cfg: &'static Config,
    label_font: Option<GameFont>,
}

impl UiShowcase {
    pub fn new(bus: &'static EventBus, cfg: &'static Config, assets: &'static Assets) -> Self {
        let label_font = assets.get_font(&TextStyle::new(FontTag::H3));
        Self {
            bus,
            cfg,
            label_font,
        }
    }
}

impl Scene for UiShowcase {
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
    }

    fn draw_ui(&self, ctx: &Context) {
        let mut ui = UI::new(self.cfg, Style::default());
        ui.begin(ctx);

        ui.rect().solid(ctx, 16.0, 16.0, 48.0, 32.0, RED, 0.0);
        ui.rect().solid(ctx, 72.0, 16.0, 48.0, 32.0, GREEN, 0.0);
        ui.rect().solid(ctx, 128.0, 16.0, 48.0, 32.0, BLUE, 0.0);
        ui.rect().solid(ctx, 184.0, 16.0, 48.0, 32.0, ORANGE, 0.0);

        ui.rect().outlined(
            ctx,
            16.0,
            56.0,
            48.0,
            32.0,
            Color::new(0.2, 0.2, 0.2, 1.0),
            WHITE,
            2.0,
            0.0,
        );
        ui.rect().outlined(
            ctx,
            72.0,
            56.0,
            48.0,
            32.0,
            Color::new(0.2, 0.2, 0.2, 1.0),
            YELLOW,
            4.0,
            0.0,
        );

        ui.rect().default_outlined(ctx, 16.0, 96.0, 48.0, 32.0);
        ui.rect().default_outlined(ctx, 72.0, 96.0, 48.0, 32.0);

        ui.rect().solid(ctx, 16.0, 140.0, 96.0, 48.0, ORANGE, 12.0);
        ui.rect().outlined(
            ctx,
            128.0,
            140.0,
            96.0,
            48.0,
            Color::new(0.2, 0.2, 0.2, 1.0),
            YELLOW,
            2.0,
            8.0,
        );

        if let Some(font) = self.label_font {
            ui.label(
                ctx,
                font,
                "Rounded boxes and a wrapped label demo",
                16.0,
                200.0,
                120.0,
            );
        }

        ui.rect().default_centered_outlined(ctx, 260.0, 120.0);
        ui.rect().centered_outlined(
            ctx,
            200.0,
            80.0,
            Color::new(0.0, 0.0, 0.0, 0.0),
            YELLOW,
            2.0,
            0.0,
        );
        ui.rect()
            .centered_solid(ctx, 120.0, 24.0, Color::new(1.0, 1.0, 1.0, 0.2), 0.0);

        ui.end();
    }
}
