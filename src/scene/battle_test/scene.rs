use super::sys_camera::CameraSystem;
use super::sys_dash::{outfit_dominant_color, DashSystem};
use super::sys_enemy_ai::EnemyAiSystem;
use super::sys_facing_lock::KnightFacingLockSystem;
use super::sys_hud::HudDrawSystem;
use super::sys_map_draw::MapDrawSystem;
use super::sys_orb::CameraOrbSystem;
use super::sys_skills_bar::SkillsBarDrawSystem;
use crate::entity::dash::Dash;
use crate::entity::factory_enemy::EnemyFactory;
use crate::entity::factory_hero::{HeroFactory, KnightCfg};
use crate::entity::factory_skills::{SkillSlotCfg, SkillsFactory};
use crate::entity::hero::HeroStats;
use crate::entity::knight::{
    Effect, EffectKind, Knight, Shield, Sword, SLASH_LIFETIME, THRUST_LIFETIME,
};
use crate::entity::player::{PlayerFactory, PlayerOne};
use crate::entity::skills::SkillIconKind;
use crate::scene::asset_preview::sys_animation::AnimationUpdateSystem;
use crate::scene::asset_preview::sys_attack_effects::{AttackEffectDrawSystem, AttackEffectSystem};
use crate::scene::asset_preview::sys_knight_controls::KnightControlSystem;
use crate::scene::asset_preview::sys_offsets::OffsetUpdateSystem;
use crate::scene::Scene;
use crate::systems::{DrawSystem, SystemAgg};
use crate::{file, font, images, shader, Animation, Assets, Config, Context, EStore};
use macroquad::prelude::*;
use pico_entity_store::store::{ChildSource, IntoChild};
use std::cell::Cell;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use tiled::{TiledMap, TiledMapCfg};

pub struct BattleTestScene {
    cfg: Rc<Config>,
    assets: Assets,
    store: Rc<EStore>,
    agg: SystemAgg,
    dash_color: Rc<Cell<Color>>,
    hud: Option<HudDrawSystem>,
    skills_bar: Option<SkillsBarDrawSystem>,
    dash_ui: Option<DashSystem>,
}

impl BattleTestScene {
    pub fn new(cfg: Rc<Config>, assets: Assets, store: Rc<EStore>) -> Self {
        let agg = SystemAgg::new();
        agg.add_update(KnightControlSystem::new(store.clone()));
        agg.add_update(KnightFacingLockSystem::new(store.clone()));
        agg.add_update(EnemyAiSystem::new(store.clone()));
        agg.add_update(AnimationUpdateSystem::new(store.clone()));
        agg.add_update(OffsetUpdateSystem::new(store.clone()));
        agg.add_update(AttackEffectSystem::new(store.clone()));
        agg.add_update(CameraOrbSystem::new(store.clone()));

        Self {
            cfg,
            assets,
            store,
            agg,
            dash_color: Rc::new(Cell::new(Color::new(1.0, 1.0, 1.0, 1.0))),
            hud: None,
            skills_bar: None,
            dash_ui: None,
        }
    }

    fn spawn_skills_widget(&self) {
        SkillsFactory::spawn(
            &self.store,
            &[
                SkillSlotCfg::icon(SkillIconKind::Sword),
                SkillSlotCfg::icon(SkillIconKind::Shield),
                SkillSlotCfg::icon(SkillIconKind::Potion),
                // SkillSlotCfg::icon(SkillIconKind::Fireball).selected(true),
                SkillSlotCfg::icon(SkillIconKind::Crossed),
                SkillSlotCfg::empty(),
            ],
        );
    }

    fn spawn_player(&self) {
        let parts = HeroFactory::new(&self.assets).create_knight(KnightCfg {
            outfit: images::Knight::Knight3,
            sword: images::Knight::Sword1,
            shield: images::Knight::Shield1,
        });

        let thrust_tex = self.assets.texture(images::Knight::ThrustGraphic);
        let swipe_tex = self.assets.texture(images::Knight::SwipeGraphic);

        let sword_rect = parts.sword_animation.rect();
        let x = sword_rect.right() - parts.sword_animation.position.x;
        let thrust_offset = vec2(x, (sword_rect.h - thrust_tex.height()) * 0.5);
        let slash_offset = vec2(x + 15.0, (sword_rect.h - swipe_tex.height()) * 0.5);

        let thrust = Effect {
            kind: EffectKind::Thrust,
            texture: thrust_tex,
            offset: thrust_offset,
            age: 0.0,
            lifetime: THRUST_LIFETIME,
            visible: false,
        };
        let slash = Effect {
            kind: EffectKind::Slash,
            texture: swipe_tex,
            offset: slash_offset,
            age: 0.0,
            lifetime: SLASH_LIFETIME,
            visible: false,
        };

        self.store
            .add(parts.shield, &[parts.shield_image.into_child()]);
        self.store.add(
            parts.sword,
            &[
                parts.sword_animation.into_child(),
                thrust.into_child(),
                slash.into_child(),
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

    fn spawn_ram_head(&self, position: Vec2) {
        let mut parts = EnemyFactory::new(&self.assets).create_ram_head();
        parts.body.position = position;
        self.store.add(parts.marker, &[parts.body.into_child()]);
    }

    fn build_map(&self) -> TiledMap {
        let tile_map = self.assets.file(file::File::TestTmx);

        let mut tile_sets = HashMap::new();
        tile_sets.insert(
            "hoi-bg-pixelated.tsx".to_string(),
            self.assets.file(file::File::HoiBgTsx),
        );
        tile_sets.insert(
            "hoi-chars.tsx".to_string(),
            self.assets.file(file::File::HoiCharsTsx),
        );

        let mut images_map = HashMap::new();
        images_map.insert(
            "hoi-bg-pixelated.png".to_string(),
            self.assets.image(images::TilesetImage::HoiBg),
        );
        images_map.insert(
            "hoi-chars.png".to_string(),
            self.assets.image(images::TilesetImage::HoiChars),
        );

        let cfg = TiledMapCfg {
            tile_map,
            tile_sets,
            images: &images_map,
            section_size: (16, 16),
        };

        TiledMap::from_config(cfg).expect("failed to build tiled map")
    }
}

impl Scene for BattleTestScene {
    fn load(&mut self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        Box::pin(async move {
            self.assets
                .preload(&[
                    &images::Knight::Knight1,
                    &images::Knight::Shield1,
                    &images::Knight::Sword1,
                    &images::Knight::Knight2,
                    &images::Knight::Shield2,
                    &images::Knight::Sword2,
                    &images::Knight::Knight3,
                    &images::Knight::Shield3,
                    &images::Knight::Sword3,
                    &images::Knight::ThrustGraphic,
                    &images::Knight::SwipeGraphic,
                    &file::File::TestTmx,
                    &file::File::HoiBgTsx,
                    &file::File::HoiCharsTsx,
                    &images::TilesetImage::HoiBg,
                    &images::TilesetImage::HoiChars,
                    &images::Enemy::RamHead,
                    &images::Knight::Face,
                    &font::Font::Pixellari,
                    &shader::Shader::DashFxVert,
                    &shader::Shader::DashAfterimageFrag,
                ])
                .await;

            self.hud = Some(HudDrawSystem::new(
                self.cfg.clone(),
                &self.assets,
                self.store.clone(),
            ));
            self.skills_bar = Some(SkillsBarDrawSystem::new(
                self.cfg.clone(),
                &self.assets,
                self.store.clone(),
            ));
            self.spawn_skills_widget();

            let map = self.build_map();
            let map_w = map.map_size.0 as f32 * map.tile_size.0 as f32;
            let map_h = map.map_size.1 as f32 * map.tile_size.1 as f32;

            self.agg
                .add_draw(MapDrawSystem::new(map, self.cfg.v_width, self.cfg.v_height));
            self.agg.add_draw(DrawSystem::new(self.store.clone()));
            self.agg
                .add_draw(AttackEffectDrawSystem::new(self.store.clone()));
            // self.agg.add_draw(CameraOrbSystem::new(self.store.clone(), 0));

            self.agg.add_update(CameraSystem::new(
                self.store.clone(),
                self.cfg.v_width,
                self.cfg.v_height,
                map_w,
                map_h,
            ));

            self.spawn_player();

            let color = {
                let Some(player) = self.store.first::<PlayerOne>() else {
                    return;
                };
                let Some(knight) = self.store.get_child::<Knight>(&player) else {
                    return;
                };
                let Some(animation) = self.store.get_child::<Animation>(&knight) else {
                    return;
                };
                outfit_dominant_color(&animation.source)
            };
            self.dash_color.set(color);

            let dash = DashSystem::new(
                self.store.clone(),
                &self.assets,
                self.dash_color.clone(),
                self.cfg.clone(),
            );
            self.agg.add_update(dash.clone());
            self.agg.add_draw(dash.clone());
            self.dash_ui = Some(dash);

            let body = vec2(200.0, 160.0);
            let anim_ref = {
                let Some(player) = self.store.first::<PlayerOne>() else {
                    return;
                };
                let Some(knight) = self.store.get_child::<Knight>(&player) else {
                    return;
                };
                let Some(animation) = self.store.get_child::<Animation>(&knight) else {
                    return;
                };
                animation.entity_ref()
            };
            self.store
                .update::<Animation, _>(&anim_ref, |a| a.position = body);
            self.spawn_ram_head(vec2(map_w * 0.5, map_h * 0.5));
        })
    }

    fn update(&mut self, ctx: &mut Context) {
        self.agg.update(ctx);
    }

    fn draw(&self, ctx: &Context) {
        self.agg.draw(ctx);
    }

    fn draw_ui(&self, ctx: &Context) {
        if let Some(hud) = &self.hud {
            hud.draw(ctx);
        }
        if let Some(skills_bar) = &self.skills_bar {
            skills_bar.draw(ctx);
        }
        if let Some(dash_ui) = &self.dash_ui {
            dash_ui.draw_ui(ctx);
        }
    }
}
