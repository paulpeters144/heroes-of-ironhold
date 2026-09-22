use super::sys_blade_barrage::BladeBarrageSkillSystem;
use super::sys_camera::CameraSystem;
use super::sys_consecration::ConsecrationSystem;
use super::sys_divine_stance::DivineStanceSystem;
use super::sys_enemy_death::EnemyDeathSystem;
use super::sys_hud::HudDrawSystem;
use super::sys_knight_attack::KnightAttackSystem;
use super::sys_knight_attack_effects::KnightAttackEffectSystem;
use super::sys_knight_attack_hit::KnightAttackHitSystem;
use super::sys_knight_controls::KnightControlSystem;
use super::sys_knight_dash::KnightDashSystem;
use super::sys_knight_offsets::KnightOffsetUpdateSystem;
use super::sys_map_draw::MapDrawSystem;
use super::sys_orb::CameraOrbSystem;
use super::sys_peon::PeonSystem;
use super::sys_ramhead_ai::RamHeadAiSystem;
use super::sys_ramhead_spawner::RamHeadSpawnerSystem;
use super::sys_shield_toss::ShieldTossSystem;
use super::sys_skill::SkillSystem;
use crate::entity::{
    AttackRect, CollisionRect, Dash, EnemyFactory, EnemyStats, HealthBar, HeroFactory, HeroStats,
    ImpactFrame, Knight, KnightCfg, LastUsedSkill, PlayerFactory, PlayerOne, RamHead, RamHeadCfg,
    Shield, Skill, SkillDirection, SkillIconKind, SkillSlotCfg, SkillsFactory, SkillsWidget, Sword,
};
use crate::events::KnightDeathEvent;
use crate::prelude::*;
use crate::scene::{ChangeSceneEvent, Scene, SceneId};
use crate::systems::{
    AnimationUpdateSystem, CollisionCircleSystem, DebugDrawSystem, DrawSystem, HealthBarSystem,
    HealthTextAnimationSystem, HitReactionSystem, KnightCombatSystem, SystemAgg, ZSortSystem,
};
use crate::{file, font, images, shader, Assets, Config};
use macroquad::prelude::*;
use macroquad::rand::gen_range;
use pico_entity_store::store::{ChildSource, IntoChild};
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use tiled::{TiledMap, TiledMapCfg};

const GAME_OVER_DELAY: f32 = 1.5;

pub struct BattleTestScene {
    cfg: Rc<Config>,
    assets: Assets,
    store: Rc<EStore>,
    bus: Rc<EventBus>,
    agg: SystemAgg,
    death_queue: Rc<RefCell<VecDeque<KnightDeathEvent>>>,
    game_over: bool,
    game_over_timer: f32,
    _subs: SubCollection,
}

impl BattleTestScene {
    pub fn new(cfg: Rc<Config>, assets: Assets, store: Rc<EStore>, bus: Rc<EventBus>) -> Self {
        let agg = SystemAgg::new();
        let death_queue = Rc::new(RefCell::new(VecDeque::new()));
        let subs = SubCollection::new();

        let death_queue_for_handler = death_queue.clone();
        subs.on::<KnightDeathEvent>(&bus, move |event: &KnightDeathEvent| {
            death_queue_for_handler.borrow_mut().push_back(event.clone());
        });

        Self {
            cfg,
            assets,
            store,
            bus,
            agg,
            death_queue,
            game_over: false,
            game_over_timer: 0.0,
            _subs: subs,
        }
    }

    fn spawn_skills_widget(&self) {
        let parts = SkillsFactory::create(&[
            SkillSlotCfg::icon(SkillIconKind::BladeBarrage).direction(SkillDirection::Up),
            SkillSlotCfg::icon(SkillIconKind::Consecration).direction(SkillDirection::Right),
            SkillSlotCfg::icon(SkillIconKind::ShieldToss).direction(SkillDirection::Down),
            SkillSlotCfg::icon(SkillIconKind::DivineStance).direction(SkillDirection::Left),
        ]);

        self.store.add(parts.widget, &[]);

        for slot in parts.slots {
            match slot.icon {
                Some(icon) => self.store.add(slot.skill, &[icon.into_child()]),
                None => self.store.add(slot.skill, &[]),
            };
            let skill_ref = self
                .store
                .all::<Skill>()
                .map(|s| s.entity_ref())
                .last()
                .expect("skill just added");
            let widget = self.store.first::<SkillsWidget>().expect("skills widget");
            self.store.add(widget, &[ChildSource::Existing(skill_ref)]);
        }

        let widget = self.store.first::<SkillsWidget>().expect("skills widget");
        self.store
            .add(widget, &[LastUsedSkill::default().into_child()]);
    }

    fn spawn_player(&self) {
        let parts = HeroFactory::create_knight(KnightCfg {
            outfit: self.assets.texture(images::Knight::Knight2),
            sword: self.assets.texture(images::Knight::Sword2),
            shield: self.assets.texture(images::Knight::Shield3),
            impact: self.assets.texture(images::Knight::Hit),
            thrust: self.assets.texture(images::Knight::ThrustGraphic),
            swipe: self.assets.texture(images::Knight::SwipeGraphic),
        });

        self.store
            .add(parts.shield, &[parts.shield_image.into_child()]);
        self.store.add(
            parts.sword,
            &[
                parts.sword_animation.into_child(),
                parts.thrust.into_child(),
                parts.slash.into_child(),
                AttackRect {
                    rects: Vec::new(),
                    visible: false,
                }
                .into_child(),
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
                parts.impact_frame.into_child(),
                parts.facing.into_child(),
                parts.collision_circle.into_child(),
                HeroStats::default().into_child(),
                Dash::knight().into_child(),
            ],
        );

        let knight = self.store.first::<Knight>().expect("knight").entity_ref();

        if let Some(impact_frame) = self
            .store
            .get_by_id::<Knight>(knight.id())
            .and_then(|k| self.store.get_child::<ImpactFrame>(&k))
        {
            self.store
                .add(impact_frame, &[parts.impact_image.into_child()]);
        }

        self.store
            .add(PlayerFactory::spawn_one(), &[ChildSource::Existing(knight)]);
    }

    fn spawn_ram_head(&self, position: Vec2) {
        let mut parts = EnemyFactory::create_ram_head(RamHeadCfg {
            body: self.assets.texture(images::Enemy::RamHead),
            impact: self.assets.texture(images::Enemy::RamHeadHit),
        });
        parts.body.position = position;
        self.store.add(
            parts.marker,
            &[
                parts.body.into_child(),
                parts.collision_circle.into_child(),
                HealthBar::default().into_child(),
                EnemyStats::default().into_child(),
                parts.impact_frame.into_child(),
                AttackRect {
                    rects: Vec::new(),
                    visible: false,
                }
                .into_child(),
            ],
        );

        let enemy_id = self
            .store
            .all::<RamHead>()
            .map(|e| e.entity_ref().id())
            .last()
            .expect("ram head just added");

        if let Some(impact_frame) = self
            .store
            .get_by_id::<RamHead>(enemy_id)
            .and_then(|e| self.store.get_child::<ImpactFrame>(&e))
        {
            self.store
                .add(impact_frame, &[parts.impact_image.into_child()]);
        }
    }

    fn spawn_collision_bounds(&self, map: &TiledMap) {
        for collide in &map.collide_statics {
            self.store.add(CollisionRect { rect: collide.0 }, &[]);
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
                    &images::Knight::Hit,
                    &images::Knight::SpinShield,
                    &file::File::TestTmx,
                    &file::File::HoiBgTsx,
                    &file::File::HoiCharsTsx,
                    &images::TilesetImage::HoiBg,
                    &images::TilesetImage::HoiChars,
                    &images::Enemy::RamHead,
                    &images::Enemy::RamHeadHit,
                    &images::Npc::HeroPeon,
                    &images::Npc::HeroPeonHit,
                    &images::Knight::Face,
                    &images::SkillIcon::BladeBarrage,
                    &images::SkillIcon::Shield,
                    &images::SkillIcon::Blast,
                    &images::Ui::SkillSlot,
                    &font::Font::Pixellari,
                    &shader::Shader::DashFxVert,
                    &shader::Shader::DashAfterimageFrag,
                    &shader::Shader::DemonDeathFrag,
                    &shader::Shader::KnightGoldenGlowFrag,
                    &shader::Shader::SpinDiscFrag,
                    &shader::Shader::DivineStanceFrag,
                ])
                .await;

            self.spawn_skills_widget();

            let map = self.build_map();
            let map_w = map.map_size.0 as f32 * map.tile_size.0 as f32;
            let map_h = map.map_size.1 as f32 * map.tile_size.1 as f32;

            self.spawn_collision_bounds(&map);

            self.spawn_player();

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

            let cx = map_w * 0.5;
            let cy = map_h * 0.5;
            let spacing = 45.0;
            let jitter = 40.0;
            for row in 0..1 {
                for col in 0..5 {
                    let offset = vec2(
                        (col as f32 - 2.0) * spacing + gen_range(-jitter, jitter),
                        (row as f32 - 0.5) * spacing + gen_range(-jitter, jitter),
                    );
                    self.spawn_ram_head(vec2(cx + offset.x, cy + offset.y));
                }
            }

            self.agg
                .add(MapDrawSystem::new(map, self.cfg.v_width, self.cfg.v_height));
            self.agg.add(DivineStanceSystem::new(
                self.store.clone(),
                self.bus.clone(),
                &self.assets,
            ));
            self.agg.add(DrawSystem::new(self.store.clone()));
            self.agg.add(ConsecrationSystem::new(
                self.store.clone(),
                self.bus.clone(),
                &self.assets,
            ));
            self.agg.add(HudDrawSystem::new(
                self.cfg.clone(),
                &self.assets,
                self.store.clone(),
            ));
            let movement_gate = Rc::new(Cell::new(true));
            self.agg.add(KnightControlSystem::with_enabled(
                self.store.clone(),
                movement_gate.clone(),
            ));
            self.agg.add(SkillSystem::new(
                self.cfg.clone(),
                &self.assets,
                self.store.clone(),
                self.bus.clone(),
                movement_gate,
            ));
            self.agg.add(BladeBarrageSkillSystem::new(
                self.store.clone(),
                self.bus.clone(),
                &self.assets,
            ));
            self.agg.add(ShieldTossSystem::new(
                self.store.clone(),
                self.bus.clone(),
                &self.assets,
            ));
            self.agg.add(KnightCombatSystem::new(
                self.store.clone(),
                self.bus.clone(),
            ));
            self.agg.add(AnimationUpdateSystem::new(self.store.clone()));
            self.agg
                .add(KnightOffsetUpdateSystem::new(self.store.clone()));
            self.agg
                .add(KnightAttackEffectSystem::new(self.store.clone()));
            self.agg.add(KnightAttackSystem::new(self.store.clone()));
            self.agg.add(KnightAttackHitSystem::new(
                self.store.clone(),
                self.bus.clone(),
            ));
            self.agg.add(CameraOrbSystem::new(self.store.clone()));
            self.agg.add(ZSortSystem::new(self.store.clone()));
            self.agg.add(HealthBarSystem::new(self.store.clone()));
            self.agg
                .add(RamHeadAiSystem::new(self.store.clone(), self.bus.clone()));
            self.agg.add(RamHeadSpawnerSystem::new(
                self.store.clone(),
                &self.assets,
                map_w,
            ));
            self.agg
                .add(PeonSystem::new(self.store.clone(), self.bus.clone(), map_w, &self.assets));
            self.agg.add(CameraSystem::new(
                self.store.clone(),
                self.cfg.v_width,
                self.cfg.v_height,
                map_w,
                map_h,
            ));

            self.agg.add(KnightDashSystem::new(
                self.store.clone(),
                &self.assets,
                self.cfg.clone(),
            ));
            // remove below to have the dash system.
            self.agg.remove::<KnightDashSystem>();

            self.agg.add(CollisionCircleSystem::new(self.store.clone()));
            self.agg.add(DebugDrawSystem::new(self.store.clone()));
            self.agg
                .add(HitReactionSystem::new(self.store.clone(), self.bus.clone()));
            self.agg.add(EnemyDeathSystem::new(
                self.store.clone(),
                self.bus.clone(),
                &self.assets,
            ));
            self.agg.add(HealthTextAnimationSystem::new(
                self.store.clone(),
                self.bus.clone(),
                &self.assets,
            ));
        })
    }

    fn update(&mut self, ctx: &mut Context) {
        if self.game_over {
            self.game_over_timer -= ctx.dt;
            if self.game_over_timer <= 0.0 {
                self.store.clear();
                self.bus.fire(&ChangeSceneEvent(SceneId::Menu));
                self.game_over = false;
            }
            return;
        }

        if self.death_queue.borrow_mut().pop_front().is_some() {
            self.game_over = true;
            self.game_over_timer = GAME_OVER_DELAY;
            return;
        }

        self.agg.update(ctx);
    }

    fn draw(&self, ctx: &Context) {
        self.agg.draw(ctx);
    }

    fn draw_ui(&self, ctx: &Context) {
        self.agg.draw_ui(ctx);
    }
}
