use crate::entity::enemy::{EnemyStats, RamHead};
use crate::entity::factory_hero::{SWORD_FRAME_COUNT, SWORD_FRAME_SIZE};
use crate::entity::knight::{Facing, Knight, KnightLock, Shield, Sword, IDLE_FRAME, THRUST_FRAME};
use crate::entity::{PlayerOne, SkillIconKind};
use crate::events::{EnemyDeathEvent, HealthChangeEvent, HitEvent, SkillCastEvent};
use crate::prelude::*;
use crate::util::{did_attack, image_data_for};
use crate::{shader, Assets};
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use macroquad::prelude::*;
use macroquad::rand::gen_range;
use pico_entity_store::entity_ref::EntityRef;
use pico_entity_store::store::IntoChild;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

/// How long the knight is held in the thrust pose for a Swords cast (seconds).
const LOCK_SECS: f32 = 0.5;
/// How far a sword travels from its spawn x before despawning (world units).
const SWORD_RANGE: f32 = 400.0; // the visible view width (cfg.v_width)
/// Swords spawned per cast.
const SWORD_COUNT: usize = 3;
/// Horizontal top travel speed of a sword (world units per second).
const SWORD_SPEED: f32 = 780.0;
/// Time (seconds) for a sword to ramp from standstill to ~95% of top speed.
const SWORD_RAMP: f32 = 0.1;
/// Time (seconds) a sword spends materializing in place before it flies:
/// it sparkles, fades in, and grows, staying stationary, then moves forward.
const SWORD_APPEAR: f32 = 0.25;
/// Flat damage applied once per enemy per sword.
const SWORD_DAMAGE: i32 = 10;
/// Time (seconds) a sword spends fading out after striking an enemy before it
/// despawns.
const SWORD_FADE: f32 = 0.35;
/// Distance (world units) a sword keeps slicing forward after striking an
/// enemy, so the blade visually cuts into the target before stopping.
const SWORD_SLICE: f32 = 30.0;
/// Base time between each sword's staggered appearance (seconds).
const SWORD_STAGGER: f32 = 0.12;
/// Random horizontal spread (beyond SPAWN_AHEAD) of a sword's spawn point.
const SPAWN_JITTER_X: f32 = 40.0;
/// Vertical gap between sword centers. Each sword's attack rect is a blade
/// strip 40% of the frame tall (~13px), so this spacing keeps the strips clear.
const SWORD_SPACING: f32 = 10.0;
/// How far the whole vertical fan of swords is lifted above the knight's center.
const SPAWN_LIFT: f32 = 30.0;
/// How far ahead of the knight's center the swords spawn.
const SPAWN_AHEAD: f32 = 30.0;
/// Magic energy color of the flying swords.
const MAGIC_COLOR: Color = Color::new(1.0, 0.8, 0.25, 1.0);
/// How many recent blade centers are kept and drawn as the speed ribbon.
/// The ribbon is bounded by the sword's own path, so it hugs the blade
/// instead of drifting far behind it.
const TRAIL_MAX: usize = 2;
/// How long the knight's golden channeling glow lasts after a cast starts
/// (seconds).
const GLOW_SECS: f32 = 0.75;

/// Per-projectile state held while a cast's swords are in flight.
struct FlyingSword {
    anim_ref: EntityRef,
    spawn_x: f32,
    delay: f32,
    fired: bool,
    appear_time: f32,
    flying_time: f32,
    /// Recent blade centers, drawn as a speed ribbon that follows the sword.
    trail: VecDeque<Vec2>,
    /// True once the sword has struck an enemy: it stops moving and fades out.
    struck: bool,
    /// Distance still left to slice forward after striking (world units).
    /// The sword carries through this much before stopping and fading.
    slice_remaining: f32,
    /// Time elapsed since the slice completed, driving the fade-out (seconds).
    fade_time: f32,
}

/// Marker for a flying sword projectile, so its `Animation` can be recovered
/// through the store after spawning.
#[derive(Clone, Debug)]
struct FlyingSwordMark;

pub struct SwordsSkillSystem {
    store: Rc<EStore>,
    bus: Rc<EventBus>,
    queue: Rc<RefCell<VecDeque<SkillCastEvent>>>,
    _subs: Rc<SubCollection>,
    swords: Vec<FlyingSword>,
    cast_dir: Vec2,
    caster: u64,
    lock_remaining: f32,
    /// Time left for the knight's golden channeling glow (seconds); starts at
    /// GLOW_SECS when the cast begins and drains each frame.
    glow_remaining: f32,
    material: Option<Material>,
    sheet_cache: Option<(Texture2D, Image)>,
}

fn load_sword_material(assets: &Assets) -> Option<Material> {
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
            warn!("sword magic shader failed to compile: {}", err);
            None
        }
    }
}

impl SwordsSkillSystem {
    pub fn new(store: Rc<EStore>, bus: Rc<EventBus>, assets: &Assets) -> Self {
        let queue = Rc::new(RefCell::new(VecDeque::new()));
        let subs = Rc::new(SubCollection::new());
        let queue_for_handler = queue.clone();
        subs.on::<SkillCastEvent>(&bus, move |event: &SkillCastEvent| {
            queue_for_handler.borrow_mut().push_back(event.clone());
        });
        Self {
            store,
            bus,
            queue,
            _subs: subs,
            swords: Vec::new(),
            cast_dir: vec2(1.0, 0.0),
            caster: 0,
            lock_remaining: 0.0,
            glow_remaining: 0.0,
            material: load_sword_material(assets),
            sheet_cache: None,
        }
    }

    fn begin_cast(&mut self, event: &SkillCastEvent) {
        // Ignore casts while a cast is still active (swords still flying).
        if !self.swords.is_empty() {
            return;
        }
        if event.kind != SkillIconKind::Sword {
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

        // Resolve facing, position, animation ref, and the knight's sword
        // texture up front so no read guard outlives the mutations below.
        let (facing, center, anim_ref, sword_tex) = {
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
            let anim_ref = self
                .store
                .get_child::<Animation>(&knight)
                .map(|a| a.entity_ref());
            let Some(sword_tex) = self
                .store
                .get_child::<Sword>(&knight)
                .and_then(|s| self.store.get_child::<Animation>(&s))
                .map(|a| a.source.clone())
            else {
                return;
            };
            (facing, center, anim_ref, sword_tex)
        };

        // Thrust pose + named lock (cosmetic; movement and attacks are held).
        if let Some(anim_ref) = anim_ref {
            self.store.update::<Animation, _>(&anim_ref, |a| {
                a.current_frame = THRUST_FRAME;
            });
        }
        let knight = self
            .store
            .get_by_id::<Knight>(knight_ref.id())
            .expect("knight alive during cast");
        self.store
            .add(knight, &[KnightLock { by: "SwordsSkill" }.into_child()]);

        // Swords travel along the knight's facing: purely horizontal.
        let dir_x = match facing {
            Facing::Left => -1.0,
            Facing::Right => 1.0,
        };
        self.cast_dir = vec2(dir_x, 0.0);
        self.caster = event.caster;
        self.lock_remaining = LOCK_SECS;
        self.glow_remaining = GLOW_SECS;

        // Spawn three swords ahead of the knight, fanned vertically around its
        // center at even spacing. X stays random; each sword stays invisible
        // until its delay elapses.
        for i in 0..SWORD_COUNT {
            let y = center.y - SPAWN_LIFT
                + (i as f32 - (SWORD_COUNT as f32 - 1.0) * 0.5) * SWORD_SPACING;
            let pos = vec2(
                center.x
                    + self.cast_dir.x
                        * (gen_range(SPAWN_AHEAD, SPAWN_AHEAD + SPAWN_JITTER_X) - 20.0),
                y,
            );
            let delay = i as f32 * SWORD_STAGGER + gen_range(0.0, SWORD_STAGGER);
            let anim = Animation {
                source: sword_tex.clone(),
                position: pos,
                frame_width: SWORD_FRAME_SIZE,
                frame_height: SWORD_FRAME_SIZE,
                frame_count: SWORD_FRAME_COUNT,
                current_frame: 1,
                running: false,
                frame_duration: 0.12,
                tint: WHITE,
                flip_x: facing == Facing::Left,
                dest_size: vec2(SWORD_FRAME_SIZE, SWORD_FRAME_SIZE),
                scale: 1.0,
                visible: false,
                z_idx: 4.0,
            };
            let area = AttackRect {
                rects: Vec::new(),
                visible: false,
            }
            .into_child();
            self.store.add(anim, &[area, FlyingSwordMark.into_child()]);
            let mark = self
                .store
                .all::<FlyingSwordMark>()
                .map(|m| m.entity_ref())
                .last()
                .expect("spawn sword");
            let anim_ref = self
                .store
                .get_by_id::<FlyingSwordMark>(mark.id())
                .and_then(|m| self.store.parent(&m))
                .expect("spawn sword");
            self.swords.push(FlyingSword {
                anim_ref,
                spawn_x: pos.x,
                delay,
                fired: false,
                appear_time: 0.0,
                flying_time: 0.0,
                trail: VecDeque::new(),
                struck: false,
                slice_remaining: 0.0,
                fade_time: 0.0,
            });
        }
    }

    fn update_cast(&mut self, dt: f32) {
        let dir = self.cast_dir;

        // Staggered appearance: a sword materializes in place once its delay
        // elapses, then starts flying after the sparkle-in phase completes.
        for sword in self.swords.iter_mut() {
            if sword.fired {
                continue;
            }
            sword.delay -= dt;
            if sword.delay <= 0.0 {
                sword.fired = true;
                // Swords stay invisible to the generic DrawSystem; this system
                // draws them itself through the magic shader (see draw()).
            }
        }

        for sword in self.swords.iter_mut() {
            if !sword.fired {
                continue;
            }
            // Sparkle-into-place: stationary materialization (no movement, no
            // active blade) before the sword flies forward.
            if sword.appear_time < SWORD_APPEAR {
                sword.appear_time += dt;
                continue;
            }
            // A struck sword slices forward only until its slice distance is
            // spent, then stops in place and fades out.
            if sword.struck && sword.slice_remaining <= 0.0 {
                continue;
            }
            sword.flying_time += dt;
            // Lerp the speed: start very slow, then ramp quickly to top speed.
            let speed = SWORD_SPEED * (1.0 - (-3.0 * sword.flying_time / SWORD_RAMP).exp());
            let Some(anim_ref) = self
                .store
                .get_by_id::<Animation>(sword.anim_ref.id())
                .map(|a| a.entity_ref())
            else {
                continue;
            };
            let Some(area_ref) = self
                .store
                .get_by_id::<Animation>(sword.anim_ref.id())
                .and_then(|a| self.store.get_child::<AttackRect>(&a))
                .map(|a| a.entity_ref())
            else {
                continue;
            };
            // Carry the slice: struck swords keep pushing forward at speed
            // until slice_remaining is consumed.
            let step = if sword.struck {
                (speed * dt).min(sword.slice_remaining)
            } else {
                speed * dt
            };
            sword.slice_remaining -= step;
            self.store.update::<Animation, _>(&anim_ref, |a| {
                a.position.x += dir.x * step;
            });
            // Once struck, the blade's attack is over: no active hit rect.
            if sword.struck {
                continue;
            }
            let rect = self
                .store
                .get_by_id::<Animation>(sword.anim_ref.id())
                .map(|a| a.rect());
            if let Some(rect) = rect {
                self.store.update::<AttackRect, _>(&area_ref, |area| {
                    let mut r = rect;
                    r.y += r.h * 0.3;
                    r.h *= 0.4;
                    area.rects = vec![r];
                    area.visible = true;
                });
                SwordsSkillSystem::spawn_fx(&mut *sword, rect.center());
            }
        }

        // Hit test through each sword's blade attack rect against the enemy's
        // pixels; a sword strikes at most one enemy, then stops and fades out.
        let mut damaged: Vec<u64> = Vec::new();
        for sword in self.swords.iter_mut() {
            if !sword.fired || sword.struck {
                continue;
            }
            let Some(area_ref) = self
                .store
                .get_by_id::<Animation>(sword.anim_ref.id())
                .and_then(|a| self.store.get_child::<AttackRect>(&a))
                .map(|a| a.entity_ref())
            else {
                continue;
            };
            let Some(rects) = self
                .store
                .get_by_id::<AttackRect>(area_ref.id())
                .map(|area| area.rects.clone())
            else {
                continue;
            };
            if !self
                .store
                .get_by_id::<AttackRect>(area_ref.id())
                .map(|area| area.visible)
                .unwrap_or(false)
            {
                continue;
            }
            let mut hit_enemy: Option<u64> = None;
            for enemy in self.store.all::<RamHead>() {
                let id = enemy.entity_ref().id();
                let Some(body) = self.store.get_child::<Animation>(&enemy) else {
                    continue;
                };
                let data = image_data_for(&body, &mut self.sheet_cache);
                if rects.iter().any(|r| did_attack(*r, &data)) {
                    hit_enemy = Some(id);
                    break;
                }
            }
            if let Some(id) = hit_enemy {
                sword.struck = true;
                sword.slice_remaining = SWORD_SLICE;
                damaged.push(id);
                // The blade's attack is over: deactivate the hit rect so the
                // fading sword can no longer damage anything.
                self.store.update::<AttackRect, _>(&area_ref, |area| {
                    area.rects = Vec::new();
                    area.visible = false;
                });
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
                    stats.hp = (stats.hp - SWORD_DAMAGE).max(0);
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
                        amount: -SWORD_DAMAGE,
                        rect,
                    });
                }
            }

            self.bus.fire(&HitEvent {
                victim: target,
                attacker: self.caster,
            });
        }

        // Despawn swords: struck swords after their fade-out completes, and the
        // rest once they have travelled SWORD_RANGE from their spawn.
        let mut i = 0;
        while i < self.swords.len() {
            let mut despawn = false;
            if self.swords[i].struck {
                // Fade only begins after the slice has carried through.
                if self.swords[i].slice_remaining <= 0.0 {
                    self.swords[i].fade_time += dt;
                    despawn = self.swords[i].fade_time >= SWORD_FADE;
                }
            } else if self.swords[i].fired {
                despawn = self
                    .store
                    .get_by_id::<Animation>(self.swords[i].anim_ref.id())
                    .map(|a| (a.position.x - self.swords[i].spawn_x).abs() >= SWORD_RANGE)
                    .unwrap_or(true);
            }
            if despawn {
                let sword = self.swords.remove(i);
                self.despawn_sword(&sword);
            } else {
                i += 1;
            }
        }
    }

    /// Record the blade's current center onto its speed ribbon. The ribbon
    /// is bounded to the sword's own recent path, so the trail always hugs
    /// the blade.
    fn spawn_fx(sword: &mut FlyingSword, center: Vec2) {
        // Push the current center; keep the ribbon short so it stays tight to
        // the sword. The segment lengths carry the speed signal.
        sword.trail.push_back(center);
        if sword.trail.len() > TRAIL_MAX {
            sword.trail.pop_front();
        }
    }

    /// Draw a sword's flight effects: a short speed ribbon built from its
    /// recent centers.
    fn draw_fx(&self, sword: &FlyingSword, flying: f32) {
        let speed_frac = flying;

        // Speed ribbon: connect the sword's recent centers. Segments stretch
        // further apart as the blade accelerates, so the ribbon itself carries
        // the speed signal while staying glued to the sword's path.
        let pts = &sword.trail;
        if pts.len() >= 2 {
            let n = pts.len();
            for i in 0..n - 1 {
                let f = (i + 1) as f32 / n as f32;
                let a = pts[i];
                let b = pts[i + 1];
                draw_line(
                    a.x,
                    a.y,
                    b.x,
                    b.y,
                    1.0 + 2.5 * f,
                    Color::new(
                        MAGIC_COLOR.r,
                        MAGIC_COLOR.g,
                        MAGIC_COLOR.b,
                        0.4 * f * speed_frac,
                    ),
                );
            }
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

    fn despawn_sword(&mut self, sword: &FlyingSword) {
        self.store.remove(&[sword.anim_ref]);
    }

    /// Draw the knight in the sword's magic style while the cast is live, so
    /// the move reads as the knight channeling the same golden energy. The
    /// body, shield, and sword are each rendered through the same
    /// knight_golden_glow material as the blades, in the same z-order as the
    /// normal draw pass.
    fn draw_knight_glow(&self) {
        if self.swords.is_empty() || self.glow_remaining <= 0.0 {
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
        material.set_uniform(
            "tint",
            vec4(MAGIC_COLOR.r, MAGIC_COLOR.g, MAGIC_COLOR.b, 1.0),
        );
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

        // Shield (z1) — hidden in the thrust pose, so respect its visibility.
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

        // Sword (z2).
        if let Some(anim) = self
            .store
            .get_by_id::<Knight>(knight_ref.id())
            .and_then(|k| self.store.get_child::<Sword>(&k))
            .and_then(|s| self.store.get_child::<Animation>(&s))
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

        gl_use_default_material();
    }
}

impl System for SwordsSkillSystem {
    fn update(&mut self, ctx: &mut Context) {
        let events: Vec<SkillCastEvent> = self.queue.borrow_mut().drain(..).collect();
        for event in events {
            self.begin_cast(&event);
        }

        if self.swords.is_empty() {
            self.lock_remaining = 0.0;
            self.glow_remaining = 0.0;
            return;
        }

        self.glow_remaining = (self.glow_remaining - ctx.dt).max(0.0);

        if self.lock_remaining > 0.0 {
            self.lock_remaining -= ctx.dt;
            if self.lock_remaining <= 0.0 {
                self.release_lock();
            }
        }

        self.update_cast(ctx.dt);
    }

    fn draw(&self, _ctx: &Context) {
        let Some(material) = &self.material else {
            return;
        };
        let trail_dir = self.cast_dir.x;

        // Golden aura around the knight while the sword cast is live.
        self.draw_knight_glow();

        for sword in self.swords.iter() {
            if !sword.fired {
                continue;
            }
            let Some(anim) = self.store.get_by_id::<Animation>(sword.anim_ref.id()) else {
                continue;
            };
            let flying = (sword.flying_time / SWORD_RAMP).min(1.0);

            let appear = (sword.appear_time / SWORD_APPEAR).min(1.0);
            let appear_ease = appear * appear * (3.0 - 2.0 * appear);
            let appear_scale = 0.25 + 0.75 * appear_ease;
            let fade = (sword.fade_time / SWORD_FADE).min(1.0);
            let fade_scale = 1.0 - fade * 0.3;
            let source = Rect::new(
                anim.current_frame as f32 * anim.frame_width,
                0.0,
                anim.frame_width,
                anim.frame_height,
            );
            let base_dest = if anim.dest_size == Vec2::ZERO {
                vec2(anim.frame_width, anim.frame_height)
            } else {
                anim.dest_size
            } * anim.scale;
            let dest_size = base_dest * appear_scale * fade_scale;
            // Grow from the blade's center rather than its top-left corner.
            let grow_offset = (base_dest - dest_size) * 0.5;
            let pos = anim.position + grow_offset;

            // Flight effects: speed ribbon.
            self.draw_fx(sword, flying);

            gl_use_material(material);
            material.set_uniform(
                "tint",
                vec4(MAGIC_COLOR.r, MAGIC_COLOR.g, MAGIC_COLOR.b, 1.0),
            );
            material.set_uniform("trail_dir", trail_dir);
            material.set_uniform("flying", flying);
            material.set_uniform("appear", appear);
            material.set_uniform("fade", fade);
            material.set_uniform("alpha", 1.0_f32);
            draw_texture_ex(
                &anim.source,
                pos.x,
                pos.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(dest_size),
                    source: Some(source),
                    flip_x: anim.flip_x,
                    ..Default::default()
                },
            );
            gl_use_default_material();
        }
    }
}
