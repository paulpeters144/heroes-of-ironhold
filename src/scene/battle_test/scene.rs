use super::sys_camera::CameraSystem;
use super::sys_facing_lock::KnightFacingLockSystem;
use super::sys_map_draw::MapDrawSystem;
use super::sys_orb::CameraOrbSystem;
use crate::entity::factory_hero::{HeroFactory, KnightCfg};
use crate::entity::knight::{
    Effect, EffectKind, Knight, Shield, Sword, SLASH_LIFETIME, THRUST_LIFETIME,
};
use crate::scene::asset_preview::sys_animation::AnimationUpdateSystem;
use crate::scene::asset_preview::sys_attack_effects::{AttackEffectDrawSystem, AttackEffectSystem};
use crate::scene::asset_preview::sys_knight_controls::KnightControlSystem;
use crate::scene::asset_preview::sys_offsets::OffsetUpdateSystem;
use crate::scene::Scene;
use crate::systems::{DrawSystem, SystemAgg};
use crate::{file, images, Animation, Assets, Config, Context, EStore};
use macroquad::prelude::*;
use pico_entity_store::store::IntoChild;
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
}

impl BattleTestScene {
    pub fn new(cfg: Rc<Config>, assets: Assets, store: Rc<EStore>) -> Self {
        let agg = SystemAgg::new();
        agg.add_update(KnightControlSystem::new(store.clone()));
        agg.add_update(KnightFacingLockSystem::new(store.clone()));
        agg.add_update(AnimationUpdateSystem::new(store.clone()));
        agg.add_update(OffsetUpdateSystem::new(store.clone()));
        agg.add_update(AttackEffectSystem::new(store.clone()));
        agg.add_update(CameraOrbSystem::new(store.clone()));

        Self {
            cfg,
            assets,
            store,
            agg,
        }
    }

    fn spawn_knight(&self) {
        let parts = HeroFactory::new(&self.assets).create_knight(KnightCfg {
            outfit: images::Knight::Knight1,
            sword: images::Knight::Sword1,
            shield: images::Knight::Shield1,
        });

        self.store
            .add(parts.shield, &[parts.shield_image.into_child()]);
        self.store
            .add(parts.sword, &[parts.sword_animation.into_child()]);

        let shield = self.store.first::<Shield>().expect("shield");
        let sword = self.store.first::<Sword>().expect("sword");
        self.store.add(
            Knight,
            &[
                parts.body.into_child(),
                shield.into_child(),
                sword.into_child(),
                parts.facing.into_child(),
            ],
        );

        self.spawn_effects();
    }

    fn spawn_effects(&self) {
        let thrust_tex = self.assets.texture(images::Knight::ThrustGraphic);
        let swipe_tex = self.assets.texture(images::Knight::SwipeGraphic);

        let (thrust_offset, slash_offset) = {
            let Some(sword) = self.store.first::<Sword>() else {
                return;
            };
            let Some(sword_anim) = self.store.get_child::<Animation>(&sword) else {
                return;
            };
            let rect = sword_anim.rect();
            let x = rect.right() - sword_anim.position.x;
            (
                vec2(x, (rect.h - thrust_tex.height()) * 0.5),
                vec2(x + 15.0, (rect.h - swipe_tex.height()) * 0.5),
            )
        };

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

        if let Some(sword) = self.store.first::<Sword>() {
            self.store
                .add(sword, &[thrust.into_child(), slash.into_child()]);
        }
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
                ])
                .await;

            let map = self.build_map();
            let map_w = map.map_size.0 as f32 * map.tile_size.0 as f32;
            let map_h = map.map_size.1 as f32 * map.tile_size.1 as f32;

            self.agg
                .add_draw(MapDrawSystem::new(map, self.cfg.v_width, self.cfg.v_height));
            self.agg.add_draw(DrawSystem::new(self.store.clone()));
            self.agg.add_draw(AttackEffectDrawSystem::new(self.store.clone()));
            self.agg.add_draw(CameraOrbSystem::new(self.store.clone()));

            self.agg.add_update(CameraSystem::new(
                self.store.clone(),
                self.cfg.v_width,
                self.cfg.v_height,
                map_w,
                map_h,
            ));

            self.spawn_knight();

            let body = vec2(200.0, 160.0);
            if let Some(knight_ref) = self.store.first::<Knight>() {
                if let Some(mut animation) = self.store.get_child_mut::<Animation>(knight_ref) {
                    animation.position = body;
                }
            }
        })
    }

    fn update(&mut self, ctx: &mut Context) {
        self.agg.update(ctx);
    }

    fn draw(&self, ctx: &Context) {
        self.agg.draw(ctx);
    }
}
