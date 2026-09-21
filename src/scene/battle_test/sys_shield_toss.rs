use crate::entity::enemy::{EnemyStats, RamHead};
use crate::entity::factory_hero::SHIELD_SIZE;
use crate::entity::knight::{Facing, Knight, KnightLock, Shield, Sword, IDLE_FRAME, SWIPE_FRAME};
use crate::entity::{PlayerOne, SkillIconKind};
use crate::events::{
    EnemyDeathEvent, HealthChangeEvent, HitEvent, SkillCastEvent, SkillCooldownEvent,
};
use crate::prelude::*;
use crate::util::{did_attack, image_data_for};
use crate::{images, shader, Assets};
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use macroquad::prelude::*;
use pico_entity_store::entity_ref::EntityRef;
use pico_entity_store::store::IntoChild;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

/// How long the knight is held in the swipe pose for a Shield Cycle cast
/// (seconds), before the lock releases and the disc continues on its own.
const STANCE_SECS: f32 = 0.3;
/// Minimum time between Shield Toss casts (seconds).
const COOLDOWN_SECS: f32 = 1.0;
/// Horizontal top travel speed of the disc (world units per second).
const DISC_SPEED: f32 = 600.0;
/// How far the disc travels from its spawn x before despawning if it never
/// detects an enemy (world units).
const DISC_RANGE: f32 = 400.0;
/// Extra distance the disc carries forward after detecting an enemy before it
/// stops and grinds (world units).
const DISC_GLIDE: f32 = 30.0;
/// Flat damage applied per damage round.
const DISC_DAMAGE: i32 = 10;
/// Number of damage rounds applied while the disc grinds against an enemy.
const DISC_ROUNDS: u32 = 5;
/// Time between consecutive damage rounds while grinding (seconds).
const DISC_TICK_SECS: f32 = 0.5;
/// How far the disc grinds forward each step while spinning against its target
/// (world units).
const DISC_GRIND_STEP_PX: f32 = 5.0;
/// Time between grind steps while the disc is digging into its target
/// (seconds); the disc keeps advancing a step this often for its whole grind.
const DISC_GRIND_STEP_SECS: f32 = 0.15;
/// How far below the knight's center the disc's starting point sits (world
/// units) — thrown down very slightly.
const DISC_SPAWN_DROP: f32 = 8.0;
/// Overall alpha of the spinning shield disc.
const DISC_ALPHA: f32 = 0.75;
/// Uniform scale applied to the disc sprite.
const DISC_SCALE: f32 = 2.0;
/// Magic energy color of the spinning shield.
const DISC_COLOR: Color = Color::new(1.0, 0.8, 0.25, 1.0);
/// How long the knight's golden channeling glow lasts after a cast starts
/// (seconds).
const GLOW_SECS: f32 = 0.75;

/// Per-disc life phase driving the fly → glide → grind behaviour.
enum DiscPhase {
    /// Travelling until an enemy is detected.
    Flying,
    /// Carrying forward the extra 30px after detection.
    Gliding,
    /// Stationary, dealing damage each round.
    Grinding,
}

/// Per-disc runtime state held by the system while the cast is active.
struct ShieldDisc {
    /// The disc's standalone `Animation` entity.
    anim_ref: EntityRef,
    /// The disc's `AttackRect` child entity.
    area_ref: EntityRef,
    /// Unit flight direction derived from the knight's facing.
    dir: Vec2,
    /// Spawn x, used to measure how far the disc has travelled.
    spawn_x: f32,
    /// fly / glide / grind.
    phase: DiscPhase,
    /// Pixels still to travel after detection (starts at DISC_GLIDE).
    glide_remaining: f32,
    /// Damage rounds remaining (starts at DISC_ROUNDS).
    rounds_left: u32,
    /// Time until the next damage round fires.
    tick_timer: f32,
    /// Accumulated time toward the next grind step (seconds); the disc advances
    /// DISC_GRIND_STEP_PX every DISC_GRIND_STEP_SECS while grinding.
    grind_timer: f32,
}

/// Marker for a thrown shield disc, so its `Animation` can be recovered
/// through the store after spawning.
#[derive(Clone, Debug)]
struct ShieldDiscMark;

pub struct ShieldTossSystem {
    store: Rc<EStore>,
    bus: Rc<EventBus>,
    queue: Rc<RefCell<VecDeque<SkillCastEvent>>>,
    _subs: Rc<SubCollection>,
    disc: Option<ShieldDisc>,
    caster: u64,
    stance_remaining: f32,
    cooldown_remaining: f32,
    cast_dir: Vec2,
    /// Time left for the knight's golden channeling glow (seconds); starts at
    /// GLOW_SECS when the cast begins and drains each frame.
    glow_remaining: f32,
    disc_tex: Texture2D,
    material: Option<Material>,
    disc_material: Option<Material>,
    sheet_cache: Option<(Texture2D, Image)>,
}

fn load_glow_material(assets: &Assets) -> Option<Material> {
    let vertex = assets.shader(shader::Shader::DashFxVert);
    let fragment = assets.shader(shader::Shader::KnightGoldenGlowFrag);

    // Custom materials don't inherit macroquad's default alpha blend, so set it
    // explicitly or transparent pixels paint opaque black over the scene.
    let alpha = BlendState::new(
        Equation::Add,
        BlendFactor::Value(BlendValue::SourceAlpha),
        BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
    );
    let keep_alpha = BlendState::new(Equation::Add, BlendFactor::Zero, BlendFactor::One);

    match load_material(
        ShaderSource::Glsl {
            vertex: &vertex,
            fragment: &fragment,
        },
        MaterialParams {
            pipeline_params: PipelineParams {
                color_blend: Some(alpha),
                alpha_blend: Some(keep_alpha),
                ..Default::default()
            },
            uniforms: vec![
                UniformDesc::new("tint", UniformType::Float4),
                UniformDesc::new("trail_dir", UniformType::Float1),
                UniformDesc::new("flying", UniformType::Float1),
                UniformDesc::new("appear", UniformType::Float1),
                UniformDesc::new("fade", UniformType::Float1),
                UniformDesc::new("alpha", UniformType::Float1),
            ],
            ..Default::default()
        },
    ) {
        Ok(material) => Some(material),
        Err(err) => {
            warn!("knight golden glow shader failed to compile: {}", err);
            None
        }
    }
}

fn load_disc_material(assets: &Assets) -> Option<Material> {
    let vertex = assets.shader(shader::Shader::DashFxVert);
    let fragment = assets.shader(shader::Shader::SpinDiscFrag);

    // Custom materials don't inherit macroquad's default alpha blend, so set it
    // explicitly or transparent pixels paint opaque black over the scene.
    let alpha = BlendState::new(
        Equation::Add,
        BlendFactor::Value(BlendValue::SourceAlpha),
        BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
    );
    let keep_alpha = BlendState::new(Equation::Add, BlendFactor::Zero, BlendFactor::One);

    match load_material(
        ShaderSource::Glsl {
            vertex: &vertex,
            fragment: &fragment,
        },
        MaterialParams {
            pipeline_params: PipelineParams {
                color_blend: Some(alpha),
                alpha_blend: Some(keep_alpha),
                ..Default::default()
            },
            uniforms: vec![
                UniformDesc::new("tint", UniformType::Float4),
                UniformDesc::new("alpha", UniformType::Float1),
            ],
            ..Default::default()
        },
    ) {
        Ok(material) => Some(material),
        Err(err) => {
            warn!("shield disc spin shader failed to compile: {}", err);
            None
        }
    }
}

impl ShieldTossSystem {
    pub fn new(store: Rc<EStore>, bus: Rc<EventBus>, assets: &Assets) -> Self {
        let queue = Rc::new(RefCell::new(VecDeque::new()));
        let subs = Rc::new(SubCollection::new());
        let queue_for_handler = queue.clone();
        subs.on::<SkillCastEvent>(&bus, move |event: &SkillCastEvent| {
            queue_for_handler.borrow_mut().push_back(event.clone());
        });
        let disc_tex = assets.texture(images::Knight::SpinShield);
        Self {
            store,
            bus,
            queue,
            _subs: subs,
            disc: None,
            caster: 0,
            stance_remaining: 0.0,
            cooldown_remaining: 0.0,
            cast_dir: vec2(1.0, 0.0),
            glow_remaining: 0.0,
            disc_tex,
            material: load_glow_material(assets),
            disc_material: load_disc_material(assets),
            sheet_cache: None,
        }
    }

    fn begin_cast(&mut self, event: &SkillCastEvent) {
        // Ignore casts for other skills, and a second cast while a disc is
        // already active or a stance is already running.
        if event.kind != SkillIconKind::ShieldToss {
            return;
        }
        if self.disc.is_some() || self.stance_remaining > 0.0 || self.cooldown_remaining > 0.0 {
            return;
        }

        let Some(knight_ref) = self
            .store
            .first::<PlayerOne>()
            .and_then(|p| self.store.get_child::<Knight>(&p))
            .map(|k| k.entity_ref())
        else {
            return;
        };

        // Defensive: SkillSystem already blocks casting while locked.
        if self
            .store
            .get_by_id::<Knight>(knight_ref.id())
            .and_then(|k| self.store.get_child::<KnightLock>(&k))
            .is_some()
        {
            return;
        }

        // Resolve facing and center up front so no read guard outlives the
        // mutations below.
        let (facing, center) = {
            let Some(knight) = self.store.get_by_id::<Knight>(knight_ref.id()) else {
                return;
            };
            let facing = self
                .store
                .get_child::<Facing>(&knight)
                .map(|f| *f)
                .unwrap_or(Facing::Right);
            let center = self
                .store
                .get_child::<Animation>(&knight)
                .map(|a| a.rect().center())
                .unwrap_or(Vec2::ZERO);
            (facing, center)
        };

        // Swipe pose + named lock (cosmetic; movement and attacks are held).
        if let Some(anim_ref) = self
            .store
            .get_by_id::<Knight>(knight_ref.id())
            .and_then(|k| self.store.get_child::<Animation>(&k))
            .map(|a| a.entity_ref())
        {
            self.store.update::<Animation, _>(&anim_ref, |a| {
                a.current_frame = SWIPE_FRAME;
            });
        }
        let knight = self
            .store
            .get_by_id::<Knight>(knight_ref.id())
            .expect("knight alive during cast");
        self.store
            .add(knight, &[KnightLock { by: "ShieldToss" }.into_child()]);

        // The disc travels along the knight's facing: purely horizontal.
        let dir_x = match facing {
            Facing::Left => -1.0,
            Facing::Right => 1.0,
        };
        self.caster = event.caster;
        self.stance_remaining = STANCE_SECS;
        self.cooldown_remaining = COOLDOWN_SECS;
        self.cast_dir = vec2(dir_x, 0.0);
        self.glow_remaining = GLOW_SECS;
        self.bus.fire(&SkillCooldownEvent {
            kind: SkillIconKind::ShieldToss,
            duration: COOLDOWN_SECS,
        });

        // The disc projects from the knight's center (thrown down very
        // slightly) and is anchored on its own center, so offset the top-left
        // position by half the scaled sprite size.
        let disc_center = vec2(center.x, center.y + DISC_SPAWN_DROP);
        let spawn = vec2(
            disc_center.x - SHIELD_SIZE * DISC_SCALE / 2.0,
            disc_center.y - SHIELD_SIZE * DISC_SCALE / 2.0,
        );
        let anim = Animation {
            source: self.disc_tex.clone(),
            position: spawn,
            frame_width: SHIELD_SIZE,
            frame_height: SHIELD_SIZE,
            frame_count: 1,
            current_frame: 0,
            running: false,
            frame_duration: 0.12,
            tint: WHITE,
            flip_x: false,
            dest_size: vec2(SHIELD_SIZE, SHIELD_SIZE),
            scale: DISC_SCALE,
            visible: false,
            z_idx: 4.0,
        };
        let area = AttackRect {
            rects: Vec::new(),
            visible: false,
        }
        .into_child();
        self.store.add(anim, &[area, ShieldDiscMark.into_child()]);
        let mark = self
            .store
            .all::<ShieldDiscMark>()
            .map(|m| m.entity_ref())
            .last()
            .expect("spawn shield disc");
        let anim_ref = self
            .store
            .get_by_id::<ShieldDiscMark>(mark.id())
            .and_then(|m| self.store.parent(&m))
            .expect("spawn shield disc");
        let area_ref = self
            .store
            .get_by_id::<Animation>(anim_ref.id())
            .and_then(|a| self.store.get_child::<AttackRect>(&a))
            .map(|a| a.entity_ref())
            .expect("disc attack rect");

        self.disc = Some(ShieldDisc {
            anim_ref,
            area_ref,
            dir: vec2(dir_x, 0.0),
            spawn_x: spawn.x,
            phase: DiscPhase::Flying,
            glide_remaining: 0.0,
            rounds_left: 0,
            tick_timer: 0.0,
            grind_timer: 0.0,
        });
    }

    fn update_cast(&mut self, dt: f32) {
        let Some(mut disc) = self.disc.take() else {
            return;
        };

        let Some(anim_ref) = self
            .store
            .get_by_id::<Animation>(disc.anim_ref.id())
            .map(|a| a.entity_ref())
        else {
            return;
        };
        let area_ref = disc.area_ref;

        let mut despawn = false;

        match disc.phase {
            DiscPhase::Flying => {
                self.store.update::<Animation, _>(&anim_ref, |a| {
                    a.position += disc.dir * DISC_SPEED * dt;
                });
                if let Some(rect) = self
                    .store
                    .get_by_id::<Animation>(anim_ref.id())
                    .map(|a| a.rect())
                {
                    self.store.update::<AttackRect, _>(&area_ref, |area| {
                        area.rects = vec![rect];
                        area.visible = true;
                    });
                }
                if self.hit_test(&disc) {
                    disc.phase = DiscPhase::Gliding;
                    disc.glide_remaining = DISC_GLIDE;
                } else {
                    let travelled = self
                        .store
                        .get_by_id::<Animation>(anim_ref.id())
                        .map(|a| (a.position.x - disc.spawn_x).abs())
                        .unwrap_or(f32::INFINITY);
                    if travelled >= DISC_RANGE {
                        despawn = true;
                    }
                }
            }
            DiscPhase::Gliding => {
                let step = (DISC_SPEED * dt).min(disc.glide_remaining);
                disc.glide_remaining -= step;
                self.store.update::<Animation, _>(&anim_ref, |a| {
                    a.position += disc.dir * step;
                });
                if let Some(rect) = self
                    .store
                    .get_by_id::<Animation>(anim_ref.id())
                    .map(|a| a.rect())
                {
                    self.store.update::<AttackRect, _>(&area_ref, |area| {
                        area.rects = vec![rect];
                        area.visible = true;
                    });
                }
                if disc.glide_remaining <= 0.0 {
                    disc.phase = DiscPhase::Grinding;
                    disc.rounds_left = DISC_ROUNDS - 1;
                    disc.tick_timer = 0.0;
                    disc.grind_timer = 0.0;
                    self.apply_round(area_ref.id());
                }
            }
            DiscPhase::Grinding => {
                // The disc keeps grinding forward in small steps while it
                // spins, digging into its target. Keep the hit rect in step
                // with the sprite so damage stays on the enemy.
                disc.grind_timer += dt;
                let mut steps = 0;
                while disc.grind_timer >= DISC_GRIND_STEP_SECS {
                    disc.grind_timer -= DISC_GRIND_STEP_SECS;
                    steps += 1;
                }
                if steps > 0 {
                    let advance = disc.dir * (DISC_GRIND_STEP_PX * steps as f32);
                    self.store.update::<Animation, _>(&anim_ref, |a| {
                        a.position += advance;
                    });
                    if let Some(rect) = self
                        .store
                        .get_by_id::<Animation>(anim_ref.id())
                        .map(|a| a.rect())
                    {
                        self.store.update::<AttackRect, _>(&area_ref, |area| {
                            area.rects = vec![rect];
                            area.visible = true;
                        });
                    }
                }
                disc.tick_timer += dt;
                let mut rounds = 0;
                while disc.tick_timer >= DISC_TICK_SECS && disc.rounds_left > 0 {
                    disc.tick_timer -= DISC_TICK_SECS;
                    disc.rounds_left -= 1;
                    rounds += 1;
                }
                for _ in 0..rounds {
                    self.apply_round(area_ref.id());
                }
                if disc.rounds_left == 0 {
                    despawn = true;
                }
            }
        }

        if despawn {
            self.store.remove(&[disc.anim_ref]);
        } else {
            self.disc = Some(disc);
        }
    }

    /// Returns true when the disc's attack rect overlaps any enemy's pixels.
    fn hit_test(&mut self, disc: &ShieldDisc) -> bool {
        let Some(rects) = self
            .store
            .get_by_id::<Animation>(disc.anim_ref.id())
            .and_then(|a| self.store.get_child::<AttackRect>(&a))
            .map(|area| area.rects.clone())
        else {
            return false;
        };
        let visible = self
            .store
            .get_by_id::<Animation>(disc.anim_ref.id())
            .and_then(|a| self.store.get_child::<AttackRect>(&a))
            .map(|area| area.visible)
            .unwrap_or(false);
        if !visible {
            return false;
        }
        for enemy in self.store.all::<RamHead>() {
            let Some(body) = self.store.get_child::<Animation>(&enemy) else {
                continue;
            };
            let data = image_data_for(&body, &mut self.sheet_cache);
            if rects.iter().any(|r| did_attack(*r, &data)) {
                return true;
            }
        }
        false
    }

    /// Re-hit-tests the disc's attack rect and applies DISC_DAMAGE to every
    /// enemy it overlaps.
    fn apply_round(&mut self, area_id: u64) {
        let Some(rects) = self
            .store
            .get_by_id::<AttackRect>(area_id)
            .map(|area| area.rects.clone())
        else {
            return;
        };
        if !self
            .store
            .get_by_id::<AttackRect>(area_id)
            .map(|area| area.visible)
            .unwrap_or(false)
        {
            return;
        }

        let mut damaged: Vec<u64> = Vec::new();
        for enemy in self.store.all::<RamHead>() {
            let id = enemy.entity_ref().id();
            let Some(body) = self.store.get_child::<Animation>(&enemy) else {
                continue;
            };
            let data = image_data_for(&body, &mut self.sheet_cache);
            if rects.iter().any(|r| did_attack(*r, &data)) {
                damaged.push(id);
            }
        }

        for target in damaged {
            let stats_ref = self
                .store
                .get_by_id::<RamHead>(target)
                .and_then(|enemy| self.store.get_child::<EnemyStats>(&enemy))
                .map(|stats| stats.entity_ref());
            if let Some(stats_ref) = stats_ref {
                let was_alive = self
                    .store
                    .get_by_id::<EnemyStats>(stats_ref.id())
                    .map(|stats| stats.hp > 0)
                    .unwrap_or(false);

                self.store.update::<EnemyStats, _>(&stats_ref, |stats| {
                    stats.hp = (stats.hp - DISC_DAMAGE).max(0);
                });

                let is_dead = self
                    .store
                    .get_by_id::<EnemyStats>(stats_ref.id())
                    .map(|stats| stats.hp <= 0)
                    .unwrap_or(false);

                if was_alive && is_dead {
                    self.bus.fire(&EnemyDeathEvent { enemy: target });
                }

                let rect = self
                    .store
                    .get_by_id::<RamHead>(target)
                    .and_then(|enemy| self.store.get_child::<Animation>(&enemy))
                    .map(|anim| anim.rect());
                if let Some(rect) = rect {
                    self.bus.fire(&HealthChangeEvent {
                        entity: target,
                        amount: -DISC_DAMAGE,
                        rect,
                    });
                }
            }

            self.bus.fire(&HitEvent {
                victim: target,
                attacker: self.caster,
            });
        }
    }

    fn release_lock(&mut self) {
        let Some(knight_ref) = self
            .store
            .first::<PlayerOne>()
            .and_then(|p| self.store.get_child::<Knight>(&p))
            .map(|k| k.entity_ref())
        else {
            return;
        };

        if let Some(lock_ref) = self
            .store
            .get_by_id::<Knight>(knight_ref.id())
            .and_then(|k| self.store.get_child::<KnightLock>(&k))
            .map(|l| l.entity_ref())
        {
            self.store.remove(&[lock_ref]);
        }

        if let Some(anim_ref) = self
            .store
            .get_by_id::<Knight>(knight_ref.id())
            .and_then(|k| self.store.get_child::<Animation>(&k))
            .map(|a| a.entity_ref())
        {
            self.store.update::<Animation, _>(&anim_ref, |a| {
                a.current_frame = IDLE_FRAME;
            });
        }
    }

    /// Draw the knight in the disc's magic style while the cast is live, so
    /// the move reads as the knight channeling the same golden energy. The
    /// body, shield, and sword are each rendered through the same
    /// knight_golden_glow material as the disc, in the same z-order as the
    /// normal draw pass.
    fn draw_knight_glow(&self) {
        if self.glow_remaining <= 0.0 {
            return;
        }
        let Some(material) = &self.material else {
            return;
        };
        let Some(knight_ref) = self
            .store
            .first::<PlayerOne>()
            .and_then(|p| self.store.get_child::<Knight>(&p))
            .map(|k| k.entity_ref())
        else {
            return;
        };

        gl_use_material(material);
        material.set_uniform("tint", vec4(DISC_COLOR.r, DISC_COLOR.g, DISC_COLOR.b, 1.0));
        material.set_uniform("trail_dir", self.cast_dir.x);
        material.set_uniform("flying", 0.4_f32);
        material.set_uniform("appear", 1.0_f32);
        material.set_uniform("fade", 0.0_f32);
        material.set_uniform("alpha", 1.0_f32);

        // Body (z0).
        if let Some(anim) = self
            .store
            .get_by_id::<Knight>(knight_ref.id())
            .and_then(|k| self.store.get_child::<Animation>(&k))
        {
            let source = Rect::new(
                anim.current_frame as f32 * anim.frame_width,
                0.0,
                anim.frame_width,
                anim.frame_height,
            );
            draw_texture_ex(
                &anim.source,
                anim.position.x,
                anim.position.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(anim.dest_size * anim.scale),
                    source: Some(source),
                    flip_x: anim.flip_x,
                    ..Default::default()
                },
            );
        }

        // Shield (z1) — hidden in the swipe pose, so respect its visibility.
        if let Some(image) = self
            .store
            .get_by_id::<Knight>(knight_ref.id())
            .and_then(|k| self.store.get_child::<Shield>(&k))
            .and_then(|s| self.store.get_child::<StaticImage>(&s))
        {
            if image.visible {
                draw_texture_ex(
                    &image.source,
                    image.position.x,
                    image.position.y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(image.size * image.scale),
                        flip_x: image.flip_x,
                        ..Default::default()
                    },
                );
            }
        }

        // Sword (z2) — hidden in the swipe pose, so respect its visibility.
        if let Some(anim) = self
            .store
            .get_by_id::<Knight>(knight_ref.id())
            .and_then(|k| self.store.get_child::<Sword>(&k))
            .and_then(|s| self.store.get_child::<Animation>(&s))
        {
            if anim.visible {
                let source = Rect::new(
                    anim.current_frame as f32 * anim.frame_width,
                    0.0,
                    anim.frame_width,
                    anim.frame_height,
                );
                draw_texture_ex(
                    &anim.source,
                    anim.position.x,
                    anim.position.y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(anim.dest_size * anim.scale),
                        source: Some(source),
                        flip_x: anim.flip_x,
                        ..Default::default()
                    },
                );
            }
        }

        gl_use_default_material();
    }
}

impl System for ShieldTossSystem {
    fn update(&mut self, ctx: &mut Context) {
        let events: Vec<SkillCastEvent> = self.queue.borrow_mut().drain(..).collect();
        for event in events {
            self.begin_cast(&event);
        }

        self.cooldown_remaining = (self.cooldown_remaining - ctx.dt).max(0.0);

        if self.stance_remaining > 0.0 {
            self.stance_remaining -= ctx.dt;
            if self.stance_remaining <= 0.0 {
                self.release_lock();
            }
        }

        self.glow_remaining = (self.glow_remaining - ctx.dt).max(0.0);

        if self.disc.is_some() {
            self.update_cast(ctx.dt);
        }
    }

    fn draw(&self, _ctx: &Context) {
        // Golden aura around the knight while the shield toss cast is live.
        self.draw_knight_glow();

        let Some(disc) = &self.disc else {
            return;
        };
        let Some(anim) = self.store.get_by_id::<Animation>(disc.anim_ref.id()) else {
            return;
        };

        let source = Rect::new(
            anim.current_frame as f32 * anim.frame_width,
            0.0,
            anim.frame_width,
            anim.frame_height,
        );
        let dest_size = if anim.dest_size == Vec2::ZERO {
            vec2(anim.frame_width, anim.frame_height)
        } else {
            anim.dest_size
        } * anim.scale;

        if let Some(material) = &self.disc_material {
            gl_use_material(material);
            material.set_uniform("tint", vec4(DISC_COLOR.r, DISC_COLOR.g, DISC_COLOR.b, 1.0));
            material.set_uniform("alpha", DISC_ALPHA);
            draw_texture_ex(
                &anim.source,
                anim.position.x,
                anim.position.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(dest_size),
                    source: Some(source),
                    flip_x: anim.flip_x,
                    ..Default::default()
                },
            );
            gl_use_default_material();
        } else {
            draw_texture_ex(
                &anim.source,
                anim.position.x,
                anim.position.y,
                Color::new(1.0, 1.0, 1.0, DISC_ALPHA),
                DrawTextureParams {
                    dest_size: Some(dest_size),
                    source: Some(source),
                    flip_x: anim.flip_x,
                    ..Default::default()
                },
            );
        }
    }
}
