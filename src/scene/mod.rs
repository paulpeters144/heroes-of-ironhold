mod factory;
mod font_showcase;
mod opening;
mod scene_one;
mod traits;
mod ui_showcase;

pub use factory::{SceneFactory, SceneId};
pub use font_showcase::FontShowcase;
pub use scene_one::SceneOne;
pub use opening::{Animation, AnimationSys, DrawSys, MenuState, MenuSys, OpeningScene};
pub use traits::Scene;
pub use ui_showcase::UiShowcase;

use crate::{Assets, Config, Context, FontTag, GameFont, TextStyle};
use macroquad::prelude::*;
use std::future::Future;
use std::pin::Pin;
use std::ptr;
use std::task::{Context as TaskContext, Poll, RawWaker, RawWakerVTable, Waker};

pub struct ChangeSceneEvent(pub SceneId);

pub struct LoadingScene {
    cfg: &'static Config,
    h1_font: Option<GameFont>,
}

impl LoadingScene {
    pub fn new(cfg: &'static Config, assets: &'static Assets) -> Self {
        let h1_font = assets.get_font(&TextStyle::new(FontTag::H3));
        Self { cfg, h1_font }
    }
}

impl Scene for LoadingScene {
    fn load(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        Box::pin(std::future::ready(()))
    }

    fn update(&mut self, _ctx: &mut Context) {}

    fn draw(&self, _ctx: &Context) {
        let cx = self.cfg.v_width * 0.5;
        let cy = self.cfg.v_height * 0.5;
        let text = "Loading...";
        if let Some(ref gf) = self.h1_font {
            let dims = measure_text(text, Some(gf.font), gf.size, 1.0);
            draw_text_ex(
                text,
                cx - dims.width * 0.5,
                cy - dims.height * 0.5 + dims.offset_y,
                TextParams {
                    font: Some(gf.font),
                    font_size: gf.size,
                    font_scale: 1.0,
                    color: gf.color,
                    ..Default::default()
                },
            );
        }
    }
}

static RAW_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(
    |_| RawWaker::new(ptr::null(), &RAW_WAKER_VTABLE),
    |_| {},
    |_| {},
    |_| {},
);

fn dummy_waker() -> Waker {
    unsafe { Waker::from_raw(RawWaker::new(ptr::null(), &RAW_WAKER_VTABLE)) }
}

pub struct SceneFuture {
    future: Pin<Box<dyn Future<Output = Box<dyn Scene>>>>,
    result: Option<Box<dyn Scene>>,
}

impl SceneFuture {
    pub fn new(future: Pin<Box<dyn Future<Output = Box<dyn Scene>>>>) -> Self {
        Self {
            future,
            result: None,
        }
    }

    pub fn is_done(&self) -> bool {
        self.result.is_some()
    }

    pub fn poll(&mut self) {
        if self.result.is_some() {
            return;
        }
        let waker = dummy_waker();
        let mut cx = TaskContext::from_waker(&waker);
        if let Poll::Ready(scene) = self.future.as_mut().poll(&mut cx) {
            self.result = Some(scene);
        }
    }

    pub fn retrieve(&mut self) -> Option<Box<dyn Scene>> {
        self.result.take()
    }
}

pub(crate) enum SceneState {
    Idle {
        scene: Box<dyn Scene>,
    },
    Transition {
        scene: Box<dyn Scene>,
        next_scene_loader: SceneFuture,
        elapsed: f32,
        min_wait: f32,
    },
}
