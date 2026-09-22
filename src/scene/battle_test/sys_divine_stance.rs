use crate::entity::knight::{DivineStance, Knight, KnightLock, IDLE_FRAME, THRUST_FRAME};
use crate::entity::{AreaRect, HeroStats, PlayerOne, SkillIconKind};
use crate::events::{
    HealthChangeEvent, SkillActiveEndEvent, SkillActiveEvent, SkillCastEvent, SkillCooldownEvent,
};
use crate::prelude::*;
use crate::{shader, Assets};
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use macroquad::prelude::*;
use macroquad::rand::gen_range;
use pico_entity_store::entity_ref::EntityRef;
use pico_entity_store::store::IntoChild;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

/// Oval zone width (world units).
const AREA_WIDTH: f32 = 150.0;
/// Oval zone height (world units).
const AREA_HEIGHT: f32 = 96.0;
/// AreaRect width (world units). Larger than the oval so the zone's bounding
/// box has room to spare.
const RECT_WIDTH: f32 = 300.0;
/// AreaRect height (world units).
const RECT_HEIGHT: f32 = 210.0;
/// Zone lifetime before it despawns (seconds).
const AREA_SECS: f32 = 10.0;
/// How long the knight is held in the thrust pose (seconds).
const LOCK_SECS: f32 = 0.5;
/// Minimum time between Divine Stance casts (seconds).
const COOLDOWN_SECS: f32 = 1.0;
/// Interval between heal ticks while the knight is inside (seconds).
const HEAL_TICK_SECS: f32 = 0.5;
/// Fraction of max HP restored per tick.
const HEAL_PCT: f32 = 0.05;

/// Magic energy color of the zone (echoes the knight's sword cast).
const MAGIC_COLOR: Color = Color::new(1.0, 0.8, 0.25, 1.0);
/// Warm gold of the appearance flash.
const SUMMON_GOLD: Color = Color::new(1.0, 0.85, 0.35, 1.0);
/// Hot white of the sparkle hearts.
const SUMMON_WHITE: Color = Color::new(1.0, 0.97, 0.85, 1.0);
/// Deep amber of the mote halos — the low end of the golden ramp.
const GOLD_DEEP: Color = Color::new(0.92, 0.62, 0.18, 1.0);
/// Pale straw-gold of the brightest motes — the high end of the ramp.
const GOLD_PALE: Color = Color::new(1.0, 0.93, 0.62, 1.0);

/// Master length of the summon flash (seconds).
const APPEAR_SECS: f32 = 0.4;
/// How long the zone fades out before it despawns (seconds).
const FADE_OUT_SECS: f32 = 0.4;
/// How long the oval grows from small to full size (seconds).
const GROW_SECS: f32 = 0.35;
/// Steady-state opacity of the golden oval.
const GLOW_ALPHA: f32 = 0.55;
/// Healing motes spawned per second while the zone is active.
const MOTES_PER_SEC: f32 = 14.0;
/// How fast a mote's sparkle twinkles (radians per second of its life).
const MOTE_TWINKLE_SPEED: f32 = 12.0;
/// How many sparkles burst out when the zone appears.
const SPARK_COUNT: usize = 16;
/// Golden sparks orbiting the oval rim while the zone is active.
const ORBITER_COUNT: usize = 5;
/// Lifetime of one heal pulse (expanding ring + light pillar), seconds.
const HEAL_PULSE_SECS: f32 = 0.6;
/// Height of the light pillar over the knight on a heal tick (world units).
const BEAM_HEIGHT: f32 = 46.0;

/// A sparkle thrown out when the zone appears.
struct Spark {
    pos: Vec2,
    vel: Vec2,
    size: f32,
}

/// A golden spark orbiting the oval's rim.
struct Orbiter {
    /// Current orbit angle (radians).
    angle: f32,
    /// Angular speed (radians per second; sign sets direction).
    speed: f32,
    size: f32,
    /// Where this orbiter sits on the gold ramp (0 = deep amber, 1 = pale).
    tone: f32,
}

/// One heal-tick flourish: an expanding ring over the zone and a light pillar
/// rising off the knight.
struct HealPulse {
    /// Life progress, 0 → 1 over HEAL_PULSE_SECS.
    t: f32,
    /// The knight's feet at the moment of the heal (beam anchor).
    pos: Vec2,
}

/// A golden healing mote that drifts up off the zone across its lifetime.
struct Mote {
    pos: Vec2,
    vel: Vec2,
    life: f32,
    max_life: f32,
    size: f32,
    /// Twinkle phase offset so motes don't blink in lockstep.
    phase: f32,
    /// Where this mote sits on the gold ramp (0 = deep amber, 1 = pale gold).
    tone: f32,
}

pub struct DivineStanceSystem {
    store: Rc<EStore>,
    bus: Rc<EventBus>,
    queue: Rc<RefCell<VecDeque<SkillCastEvent>>>,
    _subs: Rc<SubCollection>,
    /// The store entity holding the debug-visible oval bounding box, spawned
    /// while the zone is active and removed on despawn.
    area_ref: Option<EntityRef>,
    /// The `DivineStance` marker entity, spawned while the zone is active.
    marker_ref: Option<EntityRef>,
    /// Whether the zone is currently on the ground.
    active: bool,
    /// Ground center of the oval, fixed at cast time (the knight's feet).
    center: Vec2,
    /// Elapsed seconds since the zone appeared (despawns at AREA_SECS).
    area_life: f32,
    /// Time left holding the knight in the thrust pose (LOCK_SECS).
    lock_remaining: f32,
    /// Time left before the skill can be cast again (COOLDOWN_SECS).
    cooldown_remaining: f32,
    /// Time until the next heal tick (HEAL_TICK_SECS).
    heal_timer: f32,
    /// Material running the zone's procedural magic-circle shader; None if
    /// the shader failed to compile, in which case the oval falls back to a
    /// plain golden tint of the glow texture.
    glow_material: Option<Material>,
    /// Procedural soft radial glow texture. The magic-circle shader computes
    /// its own pattern from `uv` (this is only the bound texture); the
    /// no-material fallback draws it stretched into the oval footprint.
    glow_tex: Texture2D,
    /// Appearance-burst sparkles.
    sparks: Vec<Spark>,
    /// Rising healing motes.
    motes: Vec<Mote>,
    /// Time until the next mote spawns.
    mote_timer: f32,
    /// Golden sparks orbiting the oval rim.
    orbiters: Vec<Orbiter>,
    /// Expanding rings + light pillars from recent heal ticks.
    pulses: Vec<HealPulse>,
}

/// Builds the material for the zone's magic circle: a procedural golden rune
/// circle drawn entirely in the fragment shader (rotating dashed rings,
/// sweeping light shafts, pulsing core, dissolve-out), tinted with the same
/// gold as the knight's other magic. Standard alpha blend so the oval
/// composites over the ground.
fn load_glow_material(assets: &Assets) -> Option<Material> {
    let vertex = assets.shader(shader::Shader::DashFxVert);
    let fragment = assets.shader(shader::Shader::DivineStanceFrag);

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
                UniformDesc::new("appear", UniformType::Float1),
                UniformDesc::new("fade", UniformType::Float1),
                UniformDesc::new("alpha", UniformType::Float1),
            ],
            ..Default::default()
        },
    ) {
        Ok(material) => Some(material),
        Err(err) => {
            warn!("divine stance golden material failed to compile: {}", err);
            None
        }
    }
}

/// A soft white radial glow on a square texture. Drawn through the golden
/// material and stretched into the oval footprint, its alpha falloff gives the
/// zone a bright core and a soft edge.
fn make_glow_texture() -> Texture2D {
    let size: u16 = 64;
    let mut img = Image::gen_image_color(size, size, Color::new(0.0, 0.0, 0.0, 0.0));
    let half = size as f32 * 0.5;
    for y in 0..size {
        for x in 0..size {
            let px = (x as f32 + 0.5 - half) / half;
            let py = (y as f32 + 0.5 - half) / half;
            let r = (px * px + py * py).sqrt().min(1.0);
            let a = (1.0 - r).max(0.0);
            let a = a * a;
            img.set_pixel(x as u32, y as u32, Color::new(1.0, 1.0, 1.0, a));
        }
    }
    let tex = Texture2D::from_image(&img);
    tex.set_filter(FilterMode::Linear);
    tex
}

impl DivineStanceSystem {
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
            area_ref: None,
            marker_ref: None,
            active: false,
            center: Vec2::ZERO,
            area_life: 0.0,
            lock_remaining: 0.0,
            cooldown_remaining: 0.0,
            heal_timer: 0.0,
            glow_material: load_glow_material(assets),
            glow_tex: make_glow_texture(),
            sparks: Vec::new(),
            motes: Vec::new(),
            mote_timer: 0.0,
            // Fixed spread of rim orbiters: alternating directions, varied
            // speeds, sizes and gold tones so they never move in lockstep.
            orbiters: (0..ORBITER_COUNT)
                .map(|i| {
                    let f = i as f32 / ORBITER_COUNT as f32;
                    Orbiter {
                        angle: f * std::f32::consts::TAU,
                        speed: if i % 2 == 0 {
                            0.9 + f * 0.7
                        } else {
                            -(0.7 + f * 0.6)
                        },
                        size: 1.1 + f * 0.7,
                        tone: f,
                    }
                })
                .collect(),
            pulses: Vec::new(),
        }
    }

    fn begin_cast(&mut self, event: &SkillCastEvent) {
        // Ignore casts for other skills, and ignore a second cast while a zone
        // is already active or the cast lock is still running.
        if event.kind != SkillIconKind::DivineStance {
            return;
        }
        if self.active || self.lock_remaining > 0.0 || self.cooldown_remaining > 0.0 {
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

        // The zone sits on the ground under the knight: centered on the
        // bottom-center of the body rect, computed once at cast time.
        let Some(feet) = self
            .store
            .get_by_id::<Knight>(knight_ref.id())
            .and_then(|k| self.store.get_child::<Animation>(&k))
            .map(|a| {
                let r = a.rect();
                vec2(r.center().x, r.y + r.h)
            })
        else {
            return;
        };

        if let Some(anim_ref) = self
            .store
            .get_by_id::<Knight>(knight_ref.id())
            .and_then(|k| self.store.get_child::<Animation>(&k))
            .map(|a| a.entity_ref())
        {
            self.store.update::<Animation, _>(&anim_ref, |a| {
                a.current_frame = THRUST_FRAME;
            });
        }

        let knight = self
            .store
            .get_by_id::<Knight>(knight_ref.id())
            .expect("knight alive during cast");
        self.store
            .add(knight, &[KnightLock { by: "DivineStance" }.into_child()]);

        self.center = feet;
        self.active = true;
        self.area_life = 0.0;
        self.lock_remaining = LOCK_SECS;
        self.cooldown_remaining = COOLDOWN_SECS;
        self.heal_timer = HEAL_TICK_SECS;
        self.spawn_area(feet);
        self.spawn_sparks(feet);
        self.bus.fire(&SkillCooldownEvent {
            kind: SkillIconKind::DivineStance,
            duration: COOLDOWN_SECS,
        });
        self.bus.fire(&SkillActiveEvent {
            kind: SkillIconKind::DivineStance,
            duration: AREA_SECS,
        });
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

    /// Spawns the debug-visible AreaRect (a bounding box larger than the drawn
    /// oval) and the `DivineStance` marker at `center`.
    fn spawn_area(&mut self, center: Vec2) {
        let rect = Rect::new(
            center.x - RECT_WIDTH * 0.5,
            center.y - RECT_HEIGHT * 0.5,
            RECT_WIDTH,
            RECT_HEIGHT,
        );
        self.store.add(AreaRect { rect }, &[]);
        self.area_ref = self.store.all::<AreaRect>().map(|a| a.entity_ref()).last();

        self.store.add(DivineStance, &[]);
        self.marker_ref = self
            .store
            .all::<DivineStance>()
            .map(|d| d.entity_ref())
            .last();
    }

    /// Removes the AreaRect and `DivineStance` marker when the zone expires.
    fn remove_area(&mut self) {
        if let Some(area_ref) = self.area_ref.take() {
            self.store.remove(&[area_ref]);
        }
        if let Some(marker_ref) = self.marker_ref.take() {
            self.store.remove(&[marker_ref]);
        }
    }

    /// Throws a fan of golden sparkles outward when the zone appears.
    fn spawn_sparks(&mut self, center: Vec2) {
        self.sparks.clear();
        for _ in 0..SPARK_COUNT {
            let angle = gen_range(0.0, std::f32::consts::TAU);
            let speed = gen_range(30.0, 80.0);
            let size = gen_range(0.8, 1.8);
            self.sparks.push(Spark {
                pos: center,
                vel: vec2(angle.cos(), angle.sin()) * speed,
                size,
            });
        }
    }

    /// Adds a healing mote somewhere within the oval so it can rise and fade.
    fn spawn_mote(&mut self) {
        let life = gen_range(0.7, 1.4);
        let size = gen_range(0.6, 1.4);
        let rx = gen_range(-0.5, 0.5) * AREA_WIDTH;
        let ry = gen_range(-0.5, 0.5) * AREA_HEIGHT;
        self.motes.push(Mote {
            pos: self.center + vec2(rx, ry),
            vel: vec2(gen_range(-4.0, 4.0), gen_range(-28.0, -14.0)),
            life,
            max_life: life,
            size,
            phase: gen_range(0.0, std::f32::consts::TAU),
            tone: gen_range(0.0, 1.0),
        });
    }

    /// Whether the knight's body circle overlaps the oval's bounding box.
    /// A circle-vs-rect test is used so the knight's whole footprint counts.
    fn knight_in_area(&self) -> bool {
        let Some((knight_pos, knight_radius)) = self
            .store
            .first::<PlayerOne>()
            .and_then(|p| self.store.get_child::<Knight>(&p))
            .and_then(|k| {
                let circle = self.store.get_child::<CollisionCircle>(&k)?;
                let anim = self.store.get_child::<Animation>(&k)?;
                let r = anim.rect();
                let center = vec2(r.x + r.w / 2.0, r.y + r.h / 2.0);
                Some((center, circle.radius))
            })
        else {
            return false;
        };

        let oval = Rect::new(
            self.center.x - AREA_WIDTH * 0.5,
            self.center.y - AREA_HEIGHT * 0.5,
            AREA_WIDTH,
            AREA_HEIGHT,
        );

        let closest_x = knight_pos.x.clamp(oval.x, oval.x + oval.w);
        let closest_y = knight_pos.y.clamp(oval.y, oval.y + oval.h);
        let dx = knight_pos.x - closest_x;
        let dy = knight_pos.y - closest_y;
        dx * dx + dy * dy < knight_radius * knight_radius
    }

    /// Restores one 5% tick to the knight's `HeroStats` and fires a positive
    /// `HealthChangeEvent` with the knight's body rect. Skipped at full HP.
    /// Returns the knight's feet position when a heal was actually applied, so
    /// the caller can anchor a light pillar there.
    ///
    /// Every store read guard is dropped before the write-lock mutation below:
    /// the store's `RwLock` is not reentrant, so holding a `Ref` while calling
    /// `update` would deadlock.
    fn heal_knight(&self) -> Option<Vec2> {
        let knight_id = self
            .store
            .first::<PlayerOne>()
            .and_then(|p| self.store.get_child::<Knight>(&p))
            .map(|k| k.id())?;

        let (stats_ref, rect) = {
            let knight = self.store.get_by_id::<Knight>(knight_id)?;
            let stats_ref = self
                .store
                .get_child::<HeroStats>(&knight)
                .map(|s| s.entity_ref())?;
            let rect = self
                .store
                .get_child::<Animation>(&knight)
                .map(|a| a.rect())?;
            (stats_ref, rect)
        };

        let (hp, max) = self
            .store
            .get_by_id::<HeroStats>(stats_ref.id())
            .map(|s| (s.hp, s.max_hp))
            .unwrap_or((0, 0));
        if hp >= max {
            return None;
        }

        let heal = ((max as f32) * HEAL_PCT).round() as i32;
        let actual = heal.min(max - hp);
        if actual <= 0 {
            return None;
        }

        self.store
            .update::<HeroStats, _>(&stats_ref, |s| s.hp += actual);
        self.bus.fire(&HealthChangeEvent {
            entity: knight_id,
            amount: actual,
            rect,
        });

        Some(vec2(rect.center().x, rect.y + rect.h))
    }

    /// The summon flash: a soft golden bloom, one expanding elliptical ring,
    /// and the drifting sparkles, all keyed to area_life so they fire once on
    /// the cast and fade over APPEAR_SECS.
    fn draw_summon_burst(&self, t: f32) {
        if t >= 1.0 {
            return;
        }
        let fade = (1.0 - t) * (1.0 - t);
        let ease = ease_out_cubic(t);

        // Soft bloom: two faint shells with a bright center.
        draw_circle(
            self.center.x,
            self.center.y,
            30.0 * (0.8 + 0.2 * ease),
            Color::new(SUMMON_GOLD.r, SUMMON_GOLD.g, SUMMON_GOLD.b, 0.12 * fade),
        );
        draw_circle(
            self.center.x,
            self.center.y,
            16.0 * (0.8 + 0.2 * ease),
            Color::new(SUMMON_GOLD.r, SUMMON_GOLD.g, SUMMON_GOLD.b, 0.22 * fade),
        );

        // One thin elliptical ring expanding outward to match the zone.
        draw_ellipse_lines(
            self.center.x,
            self.center.y,
            AREA_WIDTH * (0.4 + 0.6 * ease),
            AREA_HEIGHT * (0.4 + 0.6 * ease),
            0.0,
            (2.0 * (1.0 - t)).max(0.5),
            Color::new(SUMMON_GOLD.r, SUMMON_GOLD.g, SUMMON_GOLD.b, 0.6 * (1.0 - t)),
        );

        // Gentle drifting sparkles, twinkling as they slow.
        for spark in self.sparks.iter() {
            let twinkle = 0.7 + 0.3 * (self.area_life * 20.0).sin();
            let a = fade * twinkle;
            draw_star_sparkle(
                spark.pos,
                spark.size * (2.0 + 0.6 * twinkle),
                Color::new(SUMMON_GOLD.r, SUMMON_GOLD.g, SUMMON_GOLD.b, a * 0.9),
            );
            draw_circle(
                spark.pos.x,
                spark.pos.y,
                spark.size * 0.6,
                Color::new(SUMMON_WHITE.r, SUMMON_WHITE.g, SUMMON_WHITE.b, a),
            );
        }
    }

    /// Golden sparks orbiting the oval's rim, twinkling as they circle.
    fn draw_orbiters(&self, w: f32, h: f32, vis: f32) {
        if vis <= 0.0 {
            return;
        }
        let rx = w * 0.5 * 0.97;
        let ry = h * 0.5 * 0.97;
        for orb in self.orbiters.iter() {
            let pos = self.center + vec2(orb.angle.cos() * rx, orb.angle.sin() * ry);
            let twinkle = 0.7 + 0.3 * (self.area_life * 9.0 + orb.angle * 3.0).sin();
            let a = vis * twinkle;
            let gold = lerp_gold(GOLD_DEEP, GOLD_PALE, orb.tone);

            draw_circle(
                pos.x,
                pos.y,
                orb.size * 2.2,
                Color::new(gold.r, gold.g, gold.b, a * 0.3),
            );
            draw_star_sparkle(
                pos,
                orb.size * 3.2,
                Color::new(SUMMON_GOLD.r, SUMMON_GOLD.g, SUMMON_GOLD.b, a),
            );
            draw_circle(
                pos.x,
                pos.y,
                orb.size * 0.6,
                Color::new(SUMMON_WHITE.r, SUMMON_WHITE.g, SUMMON_WHITE.b, a),
            );
        }
    }

    /// Heal-tick flourishes: an expanding golden ring over the zone and a
    /// soft light pillar rising off the knight, topped with a sparkle.
    fn draw_heal_pulses(&self, vis: f32) {
        if vis <= 0.0 {
            return;
        }
        for pulse in self.pulses.iter() {
            let t = pulse.t;
            let ease = ease_out_cubic(t);
            let a = (1.0 - t) * vis;

            draw_ellipse_lines(
                self.center.x,
                self.center.y,
                AREA_WIDTH * (0.3 + 0.7 * ease),
                AREA_HEIGHT * (0.3 + 0.7 * ease),
                0.0,
                2.5 * (1.0 - t) + 0.5,
                Color::new(SUMMON_GOLD.r, SUMMON_GOLD.g, SUMMON_GOLD.b, 0.7 * a),
            );

            // Light pillar: stacked strips fading with height, plus a bright
            // narrow core — a cheap vertical gradient.
            let beam_a = a * (0.5 + 0.5 * (1.0 - t));
            let h = BEAM_HEIGHT * (0.7 + 0.3 * ease);
            let strips = 8;
            let strip_h = h / strips as f32;
            for i in 0..strips {
                let f = i as f32 / strips as f32;
                let strip_a = beam_a * (1.0 - f) * (1.0 - f);
                let y = pulse.pos.y - (i + 1) as f32 * strip_h;
                draw_rectangle(
                    pulse.pos.x - 3.5,
                    y,
                    7.0,
                    strip_h + 0.5,
                    Color::new(SUMMON_GOLD.r, SUMMON_GOLD.g, SUMMON_GOLD.b, 0.35 * strip_a),
                );
                draw_rectangle(
                    pulse.pos.x - 1.0,
                    y,
                    2.0,
                    strip_h + 0.5,
                    Color::new(
                        SUMMON_WHITE.r,
                        SUMMON_WHITE.g,
                        SUMMON_WHITE.b,
                        0.6 * strip_a,
                    ),
                );
            }
            draw_star_sparkle(
                vec2(pulse.pos.x, pulse.pos.y - h),
                2.0 + 6.0 * (1.0 - t),
                Color::new(SUMMON_WHITE.r, SUMMON_WHITE.g, SUMMON_WHITE.b, a),
            );
        }
    }

    /// Rising healing motes, twinkling and fading as their life drains.
    fn draw_motes(&self, alpha: f32) {
        if alpha <= 0.0 {
            return;
        }
        for mote in self.motes.iter() {
            let life_fade = (mote.life / mote.max_life).clamp(0.0, 1.0);
            let twinkle = 0.6 + 0.4 * (mote.phase + self.area_life * MOTE_TWINKLE_SPEED).sin();
            let a = life_fade * alpha * twinkle;

            let gold = lerp_gold(GOLD_DEEP, GOLD_PALE, mote.tone);
            let hot = lerp_gold(SUMMON_GOLD, SUMMON_WHITE, mote.tone);

            // Soft halo.
            draw_circle(
                mote.pos.x,
                mote.pos.y,
                mote.size * 2.0,
                Color::new(gold.r, gold.g, gold.b, a * 0.25),
            );
            // Twinkling four-point star sparkle.
            draw_star_sparkle(
                mote.pos,
                mote.size * (2.4 + 0.8 * twinkle),
                Color::new(hot.r, hot.g, hot.b, a),
            );
            // Hot core.
            draw_circle(
                mote.pos.x,
                mote.pos.y,
                mote.size * 0.55,
                Color::new(SUMMON_WHITE.r, SUMMON_WHITE.g, SUMMON_WHITE.b, a),
            );
        }
    }
}

impl System for DivineStanceSystem {
    fn update(&mut self, ctx: &mut Context) {
        let events: Vec<SkillCastEvent> = self.queue.borrow_mut().drain(..).collect();
        for event in events {
            self.begin_cast(&event);
        }

        self.cooldown_remaining = (self.cooldown_remaining - ctx.dt).max(0.0);

        if self.lock_remaining > 0.0 {
            self.lock_remaining -= ctx.dt;
            if self.lock_remaining <= 0.0 {
                self.release_lock();
            }
        }

        if self.active {
            self.area_life += ctx.dt;

            self.heal_timer -= ctx.dt;
            if self.heal_timer <= 0.0 {
                self.heal_timer = HEAL_TICK_SECS;
                if self.knight_in_area() {
                    if let Some(feet) = self.heal_knight() {
                        self.pulses.push(HealPulse { t: 0.0, pos: feet });
                    }
                }
            }

            // Sparkles drift, slow, and stop once the flash window ends.
            for spark in self.sparks.iter_mut() {
                spark.pos += spark.vel * ctx.dt;
                spark.vel *= (1.0 - 3.0 * ctx.dt).max(0.0);
            }

            // Rim orbiters circle the oval, each at its own pace and bearing.
            for orb in self.orbiters.iter_mut() {
                orb.angle += orb.speed * ctx.dt;
            }

            // Heal pulses expand and die out.
            for pulse in self.pulses.iter_mut() {
                pulse.t += ctx.dt / HEAL_PULSE_SECS;
            }
            self.pulses.retain(|p| p.t < 1.0);

            // Motes: spawn at a steady rate, drift upward, and fade out.
            self.mote_timer -= ctx.dt;
            if self.mote_timer <= 0.0 {
                self.mote_timer = 1.0 / MOTES_PER_SEC;
                self.spawn_mote();
            }
            for mote in self.motes.iter_mut() {
                mote.pos += mote.vel * ctx.dt;
                mote.vel.x += (self.area_life * 6.0 + mote.pos.y * 0.05).sin() * 5.0 * ctx.dt;
                mote.life -= ctx.dt;
            }
            self.motes.retain(|m| m.life > 0.0);

            if self.area_life >= AREA_SECS {
                self.active = false;
                self.area_life = 0.0;
                self.sparks.clear();
                self.motes.clear();
                self.pulses.clear();
                self.remove_area();
                self.bus.fire(&SkillActiveEndEvent {
                    kind: SkillIconKind::DivineStance,
                });
            }
        }
    }

    fn draw(&self, _ctx: &Context) {
        if !self.active {
            return;
        }

        let appear = (self.area_life / APPEAR_SECS).min(1.0);
        let fade_out =
            ((self.area_life - (AREA_SECS - FADE_OUT_SECS)) / FADE_OUT_SECS).clamp(0.0, 1.0);
        let grow = (self.area_life / GROW_SECS).min(1.0);
        let grow_ease = ease_out_cubic(grow);
        let alpha = GLOW_ALPHA * (1.0 - fade_out);
        // Master visibility for full-brightness elements (particles, pulses).
        let vis = appear * (1.0 - fade_out);

        let w = AREA_WIDTH * (0.5 + 0.5 * grow_ease);
        let h = AREA_HEIGHT * (0.5 + 0.5 * grow_ease);

        self.draw_summon_burst(appear);

        // The magic circle: a procedural golden rune circle drawn entirely in
        // the fragment shader, stretched into the zone's oval footprint.
        let top_left = self.center - vec2(w, h) * 0.5;
        if let Some(material) = &self.glow_material {
            gl_use_material(material);
            material.set_uniform(
                "tint",
                vec4(MAGIC_COLOR.r, MAGIC_COLOR.g, MAGIC_COLOR.b, 1.0),
            );
            material.set_uniform("appear", appear);
            material.set_uniform("fade", fade_out);
            material.set_uniform("alpha", alpha);
            draw_texture_ex(
                &self.glow_tex,
                top_left.x,
                top_left.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(w, h)),
                    ..Default::default()
                },
            );
            gl_use_default_material();
        } else {
            draw_texture_ex(
                &self.glow_tex,
                top_left.x,
                top_left.y,
                Color::new(MAGIC_COLOR.r, MAGIC_COLOR.g, MAGIC_COLOR.b, alpha),
                DrawTextureParams {
                    dest_size: Some(vec2(w, h)),
                    ..Default::default()
                },
            );
        }

        self.draw_orbiters(w, h, vis);
        self.draw_heal_pulses(vis);
        self.draw_motes(vis);
    }
}

/// Classic ease-out cubic: fast start, gentle landing.
fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t) * (1.0 - t) * (1.0 - t)
}

/// Linear blend between two golds by `t` (0 = a, 1 = b). Keeps the alpha of `a`.
fn lerp_gold(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a,
    )
}

/// Draws a four-pointed star sparkle: a horizontal and vertical diamond lens
/// crossing at `pos`, plus a small octagonal waist for a gem-like glint. The
/// whole thing is sized by `size` (roughly the tip-to-tip length).
fn draw_star_sparkle(pos: Vec2, size: f32, color: Color) {
    let arm = size * 0.5;
    let waist = size * 0.14;

    // Horizontal lens (left-right point).
    draw_triangle(
        vec2(pos.x - arm, pos.y),
        vec2(pos.x + arm, pos.y),
        vec2(pos.x, pos.y + waist),
        color,
    );
    draw_triangle(
        vec2(pos.x - arm, pos.y),
        vec2(pos.x + arm, pos.y),
        vec2(pos.x, pos.y - waist),
        color,
    );

    // Vertical lens (up-down point).
    draw_triangle(
        vec2(pos.x, pos.y - arm),
        vec2(pos.x, pos.y + arm),
        vec2(pos.x + waist, pos.y),
        color,
    );
    draw_triangle(
        vec2(pos.x, pos.y - arm),
        vec2(pos.x, pos.y + arm),
        vec2(pos.x - waist, pos.y),
        color,
    );
}
