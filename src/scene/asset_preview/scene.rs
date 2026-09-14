use super::sys_animation::AnimationUpdateSystem;
use super::sys_knight_attack_effects::KnightAttackEffectSystem;
use super::sys_knight_controls::KnightControlSystem;
use super::sys_knight_offsets::KnightOffsetUpdateSystem;
use crate::entity::dash::Dash;
use crate::entity::factory_hero::{
    HeroFactory, KnightCfg, FRAME_SIZE, SHIELD_SIZE, SWORD_FRAME_SIZE,
};
use crate::entity::hero::HeroStats;
use crate::entity::knight::{Knight, Shield, Sword, IDLE_FRAME};
use crate::entity::player::{PlayerFactory, PlayerOne};
use crate::input::{self, Input};
use crate::scene::Scene;
use crate::systems::{DrawSystem, SystemAgg};
use crate::ui::{Outlined, Style, UI};
use crate::{images, Animation, Assets, Config, Context, EStore};
use macroquad::prelude::*;
use pico_entity_store::store::{ChildSource, IntoChild};
use std::cell::Cell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

const BOX_COUNT: usize = 3;
const OUTFITS: [images::Knight; 3] = [
    images::Knight::Knight1,
    images::Knight::Knight2,
    images::Knight::Knight3,
];
const SHIELDS: [images::Knight; 3] = [
    images::Knight::Shield1,
    images::Knight::Shield2,
    images::Knight::Shield3,
];
const SWORDS: [images::Knight; 3] = [
    images::Knight::Sword1,
    images::Knight::Sword2,
    images::Knight::Sword3,
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum FocusMode {
    Knight,
    Boxes,
}

pub struct AssetPreviewScene {
    cfg: Rc<Config>,
    assets: Assets,
    store: Rc<EStore>,
    outfit_texture: Texture2D,
    shield_texture: Texture2D,
    sword_texture: Texture2D,
    outfit_textures: [Texture2D; 3],
    shield_textures: [Texture2D; 3],
    sword_textures: [Texture2D; 3],
    thrust_tex: Texture2D,
    swipe_tex: Texture2D,
    impact_texture: Texture2D,
    knight_variant: usize,
    shield_variant: usize,
    sword_variant: usize,
    focus: usize,
    focus_mode: FocusMode,
    knight_enabled: Rc<Cell<bool>>,
    agg: SystemAgg,
}

impl AssetPreviewScene {
    pub fn new(cfg: Rc<Config>, assets: Assets, store: Rc<EStore>) -> Self {
        let agg = SystemAgg::new();
        let knight_enabled = Rc::new(Cell::new(false));
        agg.add(DrawSystem::new(store.clone()));
        agg.add(AnimationUpdateSystem::new(store.clone()));
        agg.add(KnightOffsetUpdateSystem::new(store.clone()));
        agg.add(KnightAttackEffectSystem::new(store.clone()));
        agg.add(KnightControlSystem::with_enabled(
            store.clone(),
            knight_enabled.clone(),
        ));

        Self {
            cfg,
            assets,
            store,
            outfit_texture: Texture2D::empty(),
            shield_texture: Texture2D::empty(),
            sword_texture: Texture2D::empty(),
            outfit_textures: std::array::from_fn(|_| Texture2D::empty()),
            shield_textures: std::array::from_fn(|_| Texture2D::empty()),
            sword_textures: std::array::from_fn(|_| Texture2D::empty()),
            thrust_tex: Texture2D::empty(),
            swipe_tex: Texture2D::empty(),
            impact_texture: Texture2D::empty(),
            knight_variant: 0,
            shield_variant: 0,
            sword_variant: 0,
            focus: 0,
            focus_mode: FocusMode::Boxes,
            knight_enabled,
            agg,
        }
    }

    fn box_style(&self, i: usize) -> (Color, Color, f32) {
        if self.focus_mode == FocusMode::Boxes && i == self.focus {
            (Color::new(0.35, 0.35, 0.45, 0.0), WHITE, 3.0)
        } else {
            (Color::new(0.12, 0.12, 0.18, 0.0), GRAY, 2.0)
        }
    }

    fn current_cfg(&self) -> KnightCfg {
        KnightCfg {
            outfit: self.outfit_textures[self.knight_variant].clone(),
            sword: self.sword_textures[self.sword_variant].clone(),
            shield: self.shield_textures[self.shield_variant].clone(),
            impact: self.impact_texture.clone(),
            thrust: self.thrust_tex.clone(),
            swipe: self.swipe_tex.clone(),
        }
    }

    fn spawn_player(&self, cfg: KnightCfg) {
        let parts = HeroFactory::create_knight(cfg);

        self.store
            .add(parts.shield, &[parts.shield_image.into_child()]);
        self.store.add(
            parts.sword,
            &[
                parts.sword_animation.into_child(),
                parts.thrust.into_child(),
                parts.slash.into_child(),
            ],
        );

        let shield = self.store.first::<Shield>().expect("shield");
        let sword = self.store.first::<Sword>().expect("sword");
        self.store.add(
            Knight,
            &[
                parts.body.into_child(),
                shield.into_child(),
                sword.into_child(),
                parts.facing.into_child(),
                HeroStats::default().into_child(),
                Dash::knight().into_child(),
            ],
        );

        let knight = self.store.first::<Knight>().expect("knight").entity_ref();
        self.store
            .add(PlayerFactory::spawn_one(), &[ChildSource::Existing(knight)]);
    }

    fn rebuild_player(&self) {
        let body_pos = self
            .store
            .first::<PlayerOne>()
            .and_then(|player| self.store.get_child::<Knight>(&player))
            .and_then(|knight| self.store.get_child::<Animation>(&knight))
            .map(|a| a.position);

        if let Some(player_ref) = self.store.first::<PlayerOne>().map(|p| p.entity_ref()) {
            self.store.remove(&[player_ref]);
        }

        self.spawn_player(self.current_cfg());

        if let Some(body_pos) = body_pos {
            let anim_ref = self
                .store
                .first::<PlayerOne>()
                .and_then(|player| self.store.get_child::<Knight>(&player))
                .and_then(|knight| self.store.get_child::<Animation>(&knight))
                .map(|a| a.entity_ref());
            if let Some(anim_ref) = anim_ref {
                self.store
                    .update::<Animation, _>(&anim_ref, |a| a.position = body_pos);
            }
        }
    }

    fn cycle_outfit(&mut self, delta: i32) {
        self.knight_variant =
            (self.knight_variant as i32 + delta).rem_euclid(OUTFITS.len() as i32) as usize;
        self.outfit_texture = self.outfit_textures[self.knight_variant].clone();
        self.rebuild_player();
    }

    fn cycle_shield(&mut self, delta: i32) {
        self.shield_variant =
            (self.shield_variant as i32 + delta).rem_euclid(SHIELDS.len() as i32) as usize;
        self.shield_texture = self.shield_textures[self.shield_variant].clone();
        self.rebuild_player();
    }

    fn cycle_sword(&mut self, delta: i32) {
        self.sword_variant =
            (self.sword_variant as i32 + delta).rem_euclid(SWORDS.len() as i32) as usize;
        self.sword_texture = self.sword_textures[self.sword_variant].clone();
        self.rebuild_player();
    }

    fn set_knight_idle(&self) {
        let anim_ref = self
            .store
            .first::<PlayerOne>()
            .and_then(|player| self.store.get_child::<Knight>(&player))
            .and_then(|knight| self.store.get_child::<Animation>(&knight))
            .map(|a| a.entity_ref());
        if let Some(anim_ref) = anim_ref {
            self.store
                .update::<Animation, _>(&anim_ref, |a| a.current_frame = IDLE_FRAME);
        }
    }
}

impl Scene for AssetPreviewScene {
    fn load(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        Box::pin(async move {
            self.assets
                .preload(&[
                    &images::Knight::Knight1,
                    &images::Knight::Knight2,
                    &images::Knight::Knight3,
                    &images::Knight::Shield1,
                    &images::Knight::Shield2,
                    &images::Knight::Shield3,
                    &images::Knight::Sword1,
                    &images::Knight::Sword2,
                    &images::Knight::Sword3,
                    &images::Knight::ThrustGraphic,
                    &images::Knight::SwipeGraphic,
                    &images::Knight::Hit,
                ])
                .await;

            for i in 0..3 {
                self.outfit_textures[i] = self.assets.texture(OUTFITS[i]);
                self.shield_textures[i] = self.assets.texture(SHIELDS[i]);
                self.sword_textures[i] = self.assets.texture(SWORDS[i]);
            }
            self.outfit_texture = self.outfit_textures[0].clone();
            self.shield_texture = self.shield_textures[0].clone();
            self.sword_texture = self.sword_textures[0].clone();
            self.thrust_tex = self.assets.texture(images::Knight::ThrustGraphic);
            self.swipe_tex = self.assets.texture(images::Knight::SwipeGraphic);
            self.impact_texture = self.assets.texture(images::Knight::Hit);

            self.spawn_player(self.current_cfg());

            let w = 75.0;
            let ui_left = self.cfg.v_width - w - 40.0;

            let body = vec2(
                ui_left * 0.5 - FRAME_SIZE * 0.5,
                self.cfg.v_height * 0.5 - FRAME_SIZE * 0.5,
            );
            let anim_ref = self
                .store
                .first::<PlayerOne>()
                .and_then(|player| self.store.get_child::<Knight>(&player))
                .and_then(|knight| self.store.get_child::<Animation>(&knight))
                .map(|animation| animation.entity_ref());
            if let Some(anim_ref) = anim_ref {
                self.store
                    .update::<Animation, _>(&anim_ref, |a| a.position = body);
            }
        })
    }

    fn update(&mut self, ctx: &mut Context) {
        ctx.cam_zoom = 1.0;
        ctx.cam_target = vec2(self.cfg.v_width * 0.5, self.cfg.v_height * 0.5);

        if input::down_once(Input::Jump) {
            match self.focus_mode {
                FocusMode::Boxes => {
                    self.focus_mode = FocusMode::Knight;
                }
                FocusMode::Knight => {
                    self.focus_mode = FocusMode::Boxes;
                    self.focus = 0;
                    self.set_knight_idle();
                }
            }
        }

        self.knight_enabled
            .set(self.focus_mode == FocusMode::Knight);

        if self.focus_mode != FocusMode::Knight {
            if input::down_once(Input::Up) {
                self.focus = (self.focus + BOX_COUNT - 1) % BOX_COUNT;
            }
            if input::down_once(Input::Down) {
                self.focus = (self.focus + 1) % BOX_COUNT;
            }

            match self.focus {
                0 => {
                    if input::down_once(Input::Left) {
                        self.cycle_outfit(-1);
                    }
                    if input::down_once(Input::Right) {
                        self.cycle_outfit(1);
                    }
                }
                1 => {
                    if input::down_once(Input::Left) {
                        self.cycle_shield(-1);
                    }
                    if input::down_once(Input::Right) {
                        self.cycle_shield(1);
                    }
                }
                2 => {
                    if input::down_once(Input::Left) {
                        self.cycle_sword(-1);
                    }
                    if input::down_once(Input::Right) {
                        self.cycle_sword(1);
                    }
                }
                _ => {}
            }
        }

        self.agg.update(ctx);
    }

    fn draw(&self, ctx: &Context) {
        draw_rectangle(
            0.0,
            0.0,
            self.cfg.v_width,
            self.cfg.v_height,
            Color::new(0.05, 0.05, 0.1, 1.0),
        );

        self.agg.draw(ctx);
    }

    fn draw_ui(&self, ctx: &Context) {
        let mut ui = UI::new(&self.cfg, Style::default());
        ui.begin(ctx);

        let w = 75.0;
        let h = 75.0;
        let x = self.cfg.v_width - w - 40.0;
        let box0_y = 15.0;
        let box1_y = box0_y + h + 35.0;
        let box2_y = box1_y + h + 35.0;

        draw_texture_ex(
            &self.outfit_texture,
            x + (w - FRAME_SIZE) * 0.5,
            box0_y + (h - FRAME_SIZE) * 0.5,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(FRAME_SIZE, FRAME_SIZE)),
                source: Some(Rect::new(0.0, 0.0, FRAME_SIZE, FRAME_SIZE)),
                ..Default::default()
            },
        );
        draw_texture_ex(
            &self.shield_texture,
            x + (w - SHIELD_SIZE) * 0.5,
            box1_y + (h - SHIELD_SIZE) * 0.5,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(SHIELD_SIZE, SHIELD_SIZE)),
                ..Default::default()
            },
        );
        draw_texture_ex(
            &self.sword_texture,
            x + (w - SWORD_FRAME_SIZE) * 0.5,
            box2_y + (h - SWORD_FRAME_SIZE) * 0.5,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(SWORD_FRAME_SIZE, SWORD_FRAME_SIZE)),
                source: Some(Rect::new(0.0, 0.0, SWORD_FRAME_SIZE, SWORD_FRAME_SIZE)),
                ..Default::default()
            },
        );

        let _box0 = {
            let (fill, border, thickness) = self.box_style(0);
            ui.rect().outlined(Outlined {
                ctx,
                x,
                y: box0_y,
                w,
                h,
                fill,
                border,
                thickness,
                radius: 4.0,
            })
        };
        let _box1 = {
            let (fill, border, thickness) = self.box_style(1);
            ui.rect().outlined(Outlined {
                ctx,
                x,
                y: box1_y,
                w,
                h,
                fill,
                border,
                thickness,
                radius: 4.0,
            })
        };
        let _box2 = {
            let (fill, border, thickness) = self.box_style(2);
            ui.rect().outlined(Outlined {
                ctx,
                x,
                y: box2_y,
                w,
                h,
                fill,
                border,
                thickness,
                radius: 4.0,
            })
        };

        ui.end();
    }
}
