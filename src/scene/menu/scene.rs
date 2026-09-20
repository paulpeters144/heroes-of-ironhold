use super::sys_menu_input::MenuInputSystem;
use crate::prelude::*;
use crate::scene::{ChangeSceneEvent, Scene};
use crate::systems::{AnimationUpdateSystem, DrawSystem, SystemAgg};
use crate::{font, images, Assets, Config};
use macroquad::prelude::*;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

pub struct MenuScene {
    cfg: Rc<Config>,
    assets: Assets,
    store: Rc<EStore>,
    bus: Rc<EventBus>,
    agg: SystemAgg,
    _subs: SubCollection,
}

impl MenuScene {
    pub fn new(cfg: Rc<Config>, assets: Assets, store: Rc<EStore>, bus: Rc<EventBus>) -> Self {
        let subs = SubCollection::new();
        let store_for_handler = store.clone();
        subs.on::<ChangeSceneEvent>(&bus, move |_event: &ChangeSceneEvent| {
            store_for_handler.clear();
        });
        Self {
            cfg,
            assets,
            store,
            bus,
            agg: SystemAgg::new(),
            _subs: subs,
        }
    }

    fn load_enemies(&self) {
        let ramhead = Animation {
            source: self.assets.texture(images::Scene::LargeRamhead),
            position: vec2(445.0, 110.0),
            frame_width: 112.0,
            frame_height: 128.0,
            frame_count: 5,
            current_frame: 0,
            running: false,
            frame_duration: 0.18,
            tint: WHITE,
            flip_x: false,
            dest_size: vec2(112.0, 128.0),
            scale: 1.0,
            visible: true,
            z_idx: 1.5,
        };
        self.store.add(ramhead, &[]);

        let wiz = Animation {
            source: self.assets.texture(images::Scene::RamheadWiz),
            position: vec2(455.0, 185.0),
            frame_width: 80.0,
            frame_height: 96.0,
            frame_count: 6,
            current_frame: 0,
            running: false,
            frame_duration: 0.12,
            tint: WHITE,
            flip_x: false,
            dest_size: vec2(80.0, 96.0),
            scale: 1.0,
            visible: true,
            z_idx: 1.5,
        };
        self.store.add(wiz, &[]);

        let fire_demon = Animation {
            source: self.assets.texture(images::Scene::FireDemon1),
            position: vec2(345.0, 165.0),
            frame_width: 32.0,
            frame_height: 32.0,
            frame_count: 5,
            current_frame: 0,
            running: false,
            frame_duration: 0.13,
            tint: WHITE,
            flip_x: false,
            dest_size: vec2(32.0, 32.0),
            scale: 1.0,
            visible: true,
            z_idx: 1.5,
        };
        self.store.add(fire_demon, &[]);

        let fire_demon_2 = Animation {
            source: self.assets.texture(images::Scene::FireDemon2),
            position: vec2(395.0, 290.0),
            frame_width: 48.0,
            frame_height: 48.0,
            frame_count: 7,
            current_frame: 0,
            running: false,
            frame_duration: 0.13,
            tint: WHITE,
            flip_x: false,
            dest_size: vec2(48.0, 48.0),
            scale: 1.0,
            visible: true,
            z_idx: 3.5,
        };
        self.store.add(fire_demon_2, &[]);

        let bird = Animation {
            source: self.assets.texture(images::Scene::Bird),
            position: vec2(370.0, 120.0),
            frame_width: 64.0,
            frame_height: 64.0,
            frame_count: 7,
            current_frame: 0,
            running: false,
            frame_duration: 0.12,
            tint: WHITE,
            flip_x: false,
            dest_size: vec2(64.0, 64.0),
            scale: 1.0,
            visible: true,
            z_idx: 1.5,
        };
        self.store.add(bird, &[]);

        let ramhead = Animation {
            source: self.assets.texture(images::Scene::Ramhead),
            position: vec2(390.0, 186.0),
            frame_width: 64.0,
            frame_height: 64.0,
            frame_count: 5,
            current_frame: 0,
            running: false,
            frame_duration: 0.12,
            tint: WHITE,
            flip_x: false,
            dest_size: vec2(64.0, 64.0),
            scale: 1.0,
            visible: true,
            z_idx: 1.0,
        };
        self.store.add(ramhead, &[]);

        let ram_assassin = Animation {
            source: self.assets.texture(images::Scene::AssassinDemon),
            position: vec2(365.0, 210.0),
            frame_width: 80.0,
            frame_height: 96.0,
            frame_count: 7,
            current_frame: 0,
            running: false,
            frame_duration: 0.12,
            tint: WHITE,
            flip_x: false,
            dest_size: vec2(80.0, 96.0),
            scale: 1.0,
            visible: true,
            z_idx: 1.5,
        };
        self.store.add(ram_assassin, &[]);
    }

    fn load_heroes(&self) {
        let berserk = StaticImage {
            source: self.assets.texture(images::Scene::BerserkIdle),
            position: vec2(260.0, 195.0),
            size: vec2(64.0, 64.0),
            scale: 1.0,
            tint: WHITE,
            flip_x: false,
            visible: true,
            z_idx: 2.0,
        };
        self.store.add(berserk, &[]);

        let knight = StaticImage {
            source: self.assets.texture(images::Scene::KnightIdle),
            position: vec2(280.0, 250.0),
            size: vec2(64.0, 64.0),
            scale: 1.0,
            tint: WHITE,
            flip_x: false,
            visible: true,
            z_idx: 2.1,
        };
        self.store.add(knight, &[]);

        let wizard = StaticImage {
            source: self.assets.texture(images::Scene::WizardIdle),
            position: vec2(175.0, 240.0),
            size: vec2(64.0, 64.0),
            scale: 1.0,
            tint: WHITE,
            flip_x: false,
            visible: true,
            z_idx: 1.5,
        };
        self.store.add(wizard, &[]);

        let archer = StaticImage {
            source: self.assets.texture(images::Scene::ArcherIdle),
            position: vec2(205.0, 185.0),
            size: vec2(64.0, 64.0),
            scale: 1.0,
            tint: WHITE,
            flip_x: false,
            visible: true,
            z_idx: 1.0,
        };
        self.store.add(archer, &[]);

        let assassin = StaticImage {
            source: self.assets.texture(images::Scene::AssassinIdle),
            position: vec2(225.0, 275.0),
            size: vec2(64.0, 64.0),
            scale: 1.0,
            tint: WHITE,
            flip_x: false,
            visible: true,
            z_idx: 1.5,
        };
        self.store.add(assassin, &[]);
    }

    fn load_animations(&self) {
        let torch1 = Animation {
            source: self.assets.texture(images::Scene::Torch1),
            position: vec2(565.0, 155.0),
            frame_width: 32.0,
            frame_height: 32.0,
            frame_count: 5,
            current_frame: 0,
            running: true,
            frame_duration: 0.1,
            tint: WHITE,
            flip_x: false,
            dest_size: vec2(32.0, 32.0),
            scale: 1.0,
            visible: true,
            z_idx: 1.5,
        };
        self.store.add(torch1, &[]);

        let torch2 = Animation {
            source: self.assets.texture(images::Scene::Torch2),
            position: vec2(562.0, 225.0),
            frame_width: 32.0,
            frame_height: 32.0,
            frame_count: 5,
            current_frame: 0,
            running: true,
            frame_duration: 0.1,
            tint: WHITE,
            flip_x: false,
            dest_size: vec2(32.0, 32.0),
            scale: 1.0,
            visible: true,
            z_idx: 1.5,
        };
        self.store.add(torch2, &[]);

        let flag1 = Animation {
            source: self.assets.texture(images::Scene::Flag),
            position: vec2(25.0, 105.0),
            frame_width: 32.0,
            frame_height: 32.0,
            frame_count: 7,
            current_frame: 0,
            running: true,
            frame_duration: 0.12,
            tint: WHITE,
            flip_x: false,
            dest_size: vec2(32.0, 32.0),
            scale: 1.0,
            visible: true,
            z_idx: 1.5,
        };
        self.store.add(flag1, &[]);

        let flag2 = Animation {
            source: self.assets.texture(images::Scene::Flag),
            position: vec2(300.0, 118.0),
            frame_width: 32.0,
            frame_height: 32.0,
            frame_count: 7,
            current_frame: 0,
            running: true,
            frame_duration: 0.12,
            tint: WHITE,
            flip_x: false,
            dest_size: vec2(32.0, 32.0),
            scale: 1.0,
            visible: true,
            z_idx: 1.5,
        };
        self.store.add(flag2, &[]);

        let flag3 = Animation {
            source: self.assets.texture(images::Scene::Flag),
            position: vec2(103.0, 63.0),
            frame_width: 32.0,
            frame_height: 32.0,
            frame_count: 7,
            current_frame: 2,
            running: true,
            frame_duration: 0.12,
            tint: WHITE,
            flip_x: false,
            dest_size: vec2(32.0, 32.0),
            scale: 1.0,
            visible: true,
            z_idx: 1.5,
        };
        self.store.add(flag3, &[]);
    }

    fn load_static_img(&self) {
        let background = StaticImage {
            source: self.assets.texture(images::Scene::MenuBackground),
            position: Vec2::ZERO,
            size: Vec2::new(self.cfg.v_width, self.cfg.v_height),
            scale: 1.0,
            tint: WHITE,
            flip_x: false,
            visible: true,
            z_idx: 0.0,
        };
        self.store.add(background, &[]);

        let scroll = StaticImage {
            source: self.assets.texture(images::Scene::Scroll),
            position: vec2(2.0, self.cfg.v_height - 225.0),
            size: vec2(194.0, 220.0),
            scale: 1.0,
            tint: WHITE,
            flip_x: false,
            visible: true,
            z_idx: 1.0,
        };
        self.store.add(scroll, &[]);

        let title = StaticImage {
            source: self.assets.texture(images::Scene::TitleText),
            position: vec2((self.cfg.v_width - 304.0) * 0.5, 10.0),
            size: vec2(304.0, 110.0),
            scale: 1.0,
            tint: WHITE,
            flip_x: false,
            visible: true,
            z_idx: 2.0,
        };
        self.store.add(title, &[]);
    }
}

impl Scene for MenuScene {
    fn load(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        Box::pin(async move {
            self.assets
                .preload(&[
                    &images::Scene::MenuBackground,
                    &images::Scene::Scroll,
                    &images::Scene::TitleText,
                    &images::Scene::LargeRamhead,
                    &images::Scene::RamheadWiz,
                    &images::Scene::FireDemon1,
                    &images::Scene::FireDemon2,
                    &images::Scene::Ramhead,
                    &images::Scene::AssassinDemon,
                    &images::Scene::Bird,
                    &images::Scene::BerserkIdle,
                    &images::Scene::KnightIdle,
                    &images::Scene::WizardIdle,
                    &images::Scene::AssassinIdle,
                    &images::Scene::ArcherIdle,
                    &images::Scene::Flag,
                    &images::Scene::Torch1,
                    &images::Scene::Torch2,
                    &font::Font::Pixellari,
                ])
                .await;

            self.load_static_img();
            self.load_animations();
            self.load_enemies();
            self.load_heroes();

            self.agg.add(DrawSystem::new(self.store.clone()));
            self.agg.add(AnimationUpdateSystem::new(self.store.clone()));
            self.agg.add(MenuInputSystem::new(
                &self.cfg,
                &self.assets,
                self.bus.clone(),
            ));
        })
    }

    fn update(&mut self, ctx: &mut Context) {
        ctx.cam_zoom = 1.0;
        ctx.cam_target = vec2(self.cfg.v_width * 0.5, self.cfg.v_height * 0.5);

        self.agg.update(ctx);
    }

    fn draw(&self, ctx: &Context) {
        self.agg.draw(ctx);
    }

    fn draw_ui(&self, ctx: &Context) {
        self.agg.draw_ui(ctx);
    }
}
