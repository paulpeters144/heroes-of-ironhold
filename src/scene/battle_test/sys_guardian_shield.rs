use crate::entity::knight::{GuardianShield, Knight, KnightLock, Shield, IDLE_FRAME, SWIPE_FRAME};
use crate::entity::player::PlayerOne;
use crate::entity::skills::SkillIconKind;
use crate::events::SkillCastEvent;
use crate::systems::System;
use crate::{
    images, shader, Animation, AreaRect, Assets, Context, EStore, EventBus, StaticImage,
    SubCollection,
};
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use macroquad::prelude::*;
use macroquad::rand::gen_range;
use pico_entity_store::entity_ref::EntityRef;
use pico_entity_store::store::IntoChild;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

/// How long the knight is held in the swipe pose before the shield appears
/// (seconds) — the shield appears 250ms after the cast event starts.
const STANCE_SECS: f32 = 0.25;
/// How long the shield stays visible after it appears (seconds).
const SHIELD_SECS: f32 = 75.0;
/// Time (seconds) the shield spends fading in after it appears.
const FADE_IN_SECS: f32 = 0.25;
/// Time (seconds) the shield spends fading out before it despawns.
const FADE_OUT_SECS: f32 = 0.25;
/// How far in front of the knight's center the shield sits (world units).
const SHIELD_AHEAD: f32 = 50.0;
/// The shield's maximum opacity (semi-transparent).
const SHIELD_ALPHA: f32 = 0.25;
/// Guardian shield footprint: the knight's own carried shield texture drawn
/// at 2x its native 32x32 (factory_hero::SHIELD_SIZE), so 64x64 on screen.
/// Named GUARDIAN_SHIELD_SIZE, not SHIELD_SIZE, to avoid clashing with
/// factory_hero::SHIELD_SIZE (32.0, the knight's own carried shield).
const GUARDIAN_SHIELD_SIZE: f32 = 64.0;
/// Master length of the appearance flash (seconds).
const APPEAR_SECS: f32 = 0.4;
/// How many sparkles drift out when the shield appears.
const SPARK_COUNT: usize = 12;
/// Warm gold of the appearance flash (echoes the knight's magic color).
const SUMMON_GOLD: Color = Color::new(1.0, 0.85, 0.35, 1.0);
/// Hot white of the sparkle hearts.
const SUMMON_WHITE: Color = Color::new(1.0, 0.97, 0.85, 1.0);
/// Deep amber of the mote halos — the low end of the golden ramp.
const GOLD_DEEP: Color = Color::new(0.92, 0.62, 0.18, 1.0);
/// Pale straw-gold of the brightest motes — the high end of the ramp.
const GOLD_PALE: Color = Color::new(1.0, 0.93, 0.62, 1.0);
/// How fast a mote's sparkle twinkles (radians per second of its life).
const MOTE_TWINKLE_SPEED: f32 = 14.0;
/// How long the shield grows from small to full size after appearing.
const GROW_SECS: f32 = 0.3;
/// Motes spawned per second while the shield is active.
const MOTES_PER_SEC: f32 = 10.0;

/// Standard macroquad vertex shader (same as the built-in default).
const ADDITIVE_VERT: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;

varying lowp vec2 uv;
varying lowp vec4 color;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    color = color0 / 255.0;
    uv = texcoord;
}"#;

/// Pass-through fragment shader; the glow comes from the pipeline's additive
/// blend state, not the shader.
const ADDITIVE_FRAG: &str = r#"#version 100
varying lowp vec2 uv;
varying lowp vec4 color;

uniform sampler2D Texture;

void main() {
    gl_FragColor = color * texture2D(Texture, uv);
}"#;

/// A sparkle thrown out when the shield appears.
struct Spark {
    pos: Vec2,
    vel: Vec2,
    size: f32,
}

/// A mote that drifts up off the shield across its lifetime, giving it a slow
/// magical shimmer while active. Each mote twinkles and varies in gold tone.
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

pub struct GuardianShieldSystem {
    store: Rc<EStore>,
    queue: Rc<RefCell<VecDeque<SkillCastEvent>>>,
    _subs: Rc<SubCollection>,
    stance_remaining: f32,
    shield_active: bool,
    shield_life: f32,
    /// The store entity holding the shield's debug-visible area rect, spawned
    /// while the shield is active and removed on despawn.
    area_ref: Option<EntityRef>,
    /// The `GuardianShield` marker entity, spawned while the shield is active
    /// so combat can boost the knight's armor, removed on despawn.
    shield_ref: Option<EntityRef>,
    shield_tex: Texture2D,
    sparks: Vec<Spark>,
    motes: Vec<Mote>,
    mote_timer: f32,
    /// Additive-blend material for the glow pass; None if the shader failed
    /// to compile, in which case the effect falls back to plain alpha draws.
    glow_material: Option<Material>,
    /// Golden material for the shield sprite itself; None if the shader failed
    /// to compile, in which case the sprite is drawn with its plain texture.
    shield_material: Option<Material>,
}

impl GuardianShieldSystem {
    pub fn new(store: Rc<EStore>, bus: Rc<EventBus>, assets: &Assets) -> Self {
        let queue = Rc::new(RefCell::new(VecDeque::new()));
        let subs = Rc::new(SubCollection::new());
        let queue_for_handler = queue.clone();
        subs.on::<SkillCastEvent>(&bus, move |event: &SkillCastEvent| {
            queue_for_handler.borrow_mut().push_back(event.clone());
        });
        let shield_tex = store
            .first::<Shield>()
            .and_then(|shield| store.get_child::<StaticImage>(&shield))
            .map(|img| img.source.clone())
            .unwrap_or_else(|| assets.texture(images::Knight::Shield1));
        Self {
            store,
            queue,
            _subs: subs,
            stance_remaining: 0.0,
            shield_active: false,
            shield_life: 0.0,
            area_ref: None,
            shield_ref: None,
            shield_tex,
            sparks: Vec::new(),
            motes: Vec::new(),
            mote_timer: 0.0,
            glow_material: load_glow_material(),
            shield_material: load_shield_material(assets),
        }
    }

    fn begin_cast(&mut self, event: &SkillCastEvent) {
        // Ignore casts for other skills, and ignore a second cast while a
        // shield is already active or a stance is already running.
        if event.kind != SkillIconKind::Shield {
            return;
        }
        if self.shield_active || self.stance_remaining > 0.0 {
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

        if let Some(anim_ref) = self
            .store
            .get_by_id::<Knight>(knight_ref.id())
            .and_then(|k| self.store.get_child::<Animation>(&k))
            .map(|a| a.entity_ref())
        {
            self.enter_swipe_pose(&anim_ref);
        }

        let knight = self
            .store
            .get_by_id::<Knight>(knight_ref.id())
            .expect("knight alive during cast");
        self.store.add(
            knight,
            &[KnightLock {
                by: "GuardianShield",
            }
            .into_child()],
        );

        self.stance_remaining = STANCE_SECS;
        self.shield_active = false;
        self.shield_life = 0.0;
    }

    /// Sets the knight's animation to SWIPE_FRAME (frame 2, the swipe pose —
    /// which also hides the knight's own sword and shield).
    fn enter_swipe_pose(&mut self, anim_ref: &EntityRef) {
        self.store.update::<Animation, _>(anim_ref, |a| {
            a.current_frame = SWIPE_FRAME;
        });
    }

    /// Returns where the shield should be drawn: the knight's current center
    /// plus SHIELD_AHEAD to the right, regardless of facing. Re-read each frame
    /// so the shield tracks the knight.
    fn shield_position(&self, knight_center: Vec2) -> Vec2 {
        knight_center + vec2(SHIELD_AHEAD, 0.0)
    }

    /// Throws a few gentle sparkles outward when the shield appears.
    fn spawn_sparks(&mut self, center: Vec2) {
        self.sparks.clear();
        for _ in 0..SPARK_COUNT {
            let angle = gen_range(0.0, std::f32::consts::TAU);
            let speed = gen_range(25.0, 70.0);
            let size = gen_range(0.8, 1.8);
            self.sparks.push(Spark {
                pos: center,
                vel: vec2(angle.cos(), angle.sin()) * speed,
                size,
            });
        }
    }

    /// Adds a mote somewhere over the shield's face so it can rise and fade.
    fn spawn_mote(&mut self, shield_pos: Vec2) {
        let life = gen_range(0.8, 1.6);
        let size = gen_range(0.7, 1.6);
        self.motes.push(Mote {
            pos: shield_pos + vec2(gen_range(-24.0, 24.0), gen_range(-28.0, 18.0)),
            vel: vec2(gen_range(-4.0, 4.0), gen_range(-24.0, -12.0)),
            life,
            max_life: life,
            size,
            phase: gen_range(0.0, std::f32::consts::TAU),
            tone: gen_range(0.0, 1.0),
        });
    }

    /// The knight's current body center, or None if the knight is gone.
    fn knight_center(&self) -> Option<Vec2> {
        self.store
            .first::<PlayerOne>()
            .and_then(|p| self.store.get_child::<Knight>(&p))
            .and_then(|k| self.store.get_child::<Animation>(&k))
            .map(|a| a.rect().center())
    }

    /// The shield's current world position, or None if the knight is gone.
    fn shield_pos(&self) -> Option<Vec2> {
        let knight_ref = self
            .store
            .first::<PlayerOne>()
            .and_then(|p| self.store.get_child::<Knight>(&p))
            .map(|k| k.entity_ref())?;
        let knight = self.store.get_by_id::<Knight>(knight_ref.id())?;
        let center = self
            .store
            .get_child::<Animation>(&knight)
            .map(|a| a.rect().center())?;
        Some(self.shield_position(center))
    }

    /// The shield's full-size footprint as a world rect.
    fn shield_rect(&self) -> Rect {
        let size = GUARDIAN_SHIELD_SIZE;
        let top_left = self.shield_pos().unwrap_or(Vec2::ZERO) - vec2(size, size) * 0.5;
        Rect::new(top_left.x, top_left.y, size, size)
    }

    /// Spawns the debug-visible AreaRect entity when the shield appears.
    fn spawn_area_rect(&mut self) {
        let rect = self.shield_rect();
        self.store.add(AreaRect { rect }, &[]);
        self.area_ref = self.store.all::<AreaRect>().map(|a| a.entity_ref()).last();

        // The marker signals combat to boost the knight's armor while up.
        self.store.add(GuardianShield, &[]);
        self.shield_ref = self
            .store
            .all::<GuardianShield>()
            .map(|g| g.entity_ref())
            .last();
    }

    /// Moves the AreaRect with the shield each frame while it's active.
    fn update_area_rect(&mut self) {
        let rect = self.shield_rect();
        if let Some(area_ref) = &self.area_ref {
            self.store
                .update::<AreaRect, _>(area_ref, |a| a.rect = rect);
        }
    }

    /// Removes the AreaRect entity when the shield despawns.
    fn remove_area_rect(&mut self) {
        if let Some(area_ref) = self.area_ref.take() {
            self.store.remove(&[area_ref]);
        }
        if let Some(shield_ref) = self.shield_ref.take() {
            self.store.remove(&[shield_ref]);
        }
    }

    /// Draws the shield sprite in front of the knight when shield_active is
    /// true. On appearance: a soft additive glow bloom, one thin expanding
    /// ring, and a few drifting sparkles, all fading over APPEAR_SECS, with
    /// the shield sprite itself growing in over GROW_SECS and alpha-ramping
    /// over FADE_IN_SECS / FADE_OUT_SECS as before.
    fn draw_shield(&self) {
        if !self.shield_active {
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
        let Some(knight) = self.store.get_by_id::<Knight>(knight_ref.id()) else {
            return;
        };
        let center = self
            .store
            .get_child::<Animation>(&knight)
            .map(|a| a.rect().center())
            .unwrap_or(Vec2::ZERO);

        let pos = self.shield_position(center);

        let fade_in = (self.shield_life / FADE_IN_SECS).min(1.0);
        let fade_out =
            ((self.shield_life - (SHIELD_SECS - FADE_OUT_SECS)) / FADE_OUT_SECS).clamp(0.0, 1.0);
        let alpha = SHIELD_ALPHA * fade_in * (1.0 - fade_out);

        // Appearance glow behind the shield (additive when available).
        if let Some(material) = self.glow_material.as_ref() {
            gl_use_material(material);
        }
        self.draw_appear_glow(pos);
        self.draw_life_glow(pos, alpha);
        if self.glow_material.is_some() {
            gl_use_default_material();
        }

        // The shield sprite itself, growing in gently. Drawn through the
        // golden material when available, else its plain texture.
        let grow = (self.shield_life / GROW_SECS).min(1.0);
        let size = GUARDIAN_SHIELD_SIZE * (0.6 + 0.4 * ease_out_cubic(grow));

        let top_left = pos - vec2(size, size) * 0.5;
        if let Some(material) = self.shield_material.as_ref() {
            material.set_uniform(
                "tint",
                vec4(SUMMON_GOLD.r, SUMMON_GOLD.g, SUMMON_GOLD.b, 1.0),
            );
            // The shield is a static ward, not a flying blade: no trail, no
            // appearance burst, no shatter — just the golden tint, rim, and
            // pulse from the shared shader, scaled by the shield's alpha.
            material.set_uniform("trail_dir", 1.0_f32);
            material.set_uniform("flying", 0.0_f32);
            material.set_uniform("appear", 1.0_f32);
            material.set_uniform("fade", 0.0_f32);
            material.set_uniform("alpha", alpha);
            gl_use_material(material);
            draw_texture_ex(
                &self.shield_tex,
                top_left.x,
                top_left.y,
                Color::new(1.0, 1.0, 1.0, 1.0),
                DrawTextureParams {
                    dest_size: Some(vec2(size, size)),
                    ..Default::default()
                },
            );
            gl_use_default_material();
        } else {
            draw_texture_ex(
                &self.shield_tex,
                top_left.x,
                top_left.y,
                Color::new(1.0, 1.0, 1.0, alpha),
                DrawTextureParams {
                    dest_size: Some(vec2(size, size)),
                    ..Default::default()
                },
            );
        }
    }

    /// The appearance flash: a soft gold bloom, a single thin expanding ring,
    /// and the drifting sparkles, all keyed to shield_life so they fire once
    /// on the summon and fade over APPEAR_SECS.
    fn draw_appear_glow(&self, pos: Vec2) {
        let t = (self.shield_life / APPEAR_SECS).clamp(0.0, 1.0);
        if t >= 1.0 {
            return;
        }
        let fade = (1.0 - t) * (1.0 - t);
        let ease = ease_out_cubic(t);

        // Soft bloom: two faint shells with a small bright center.
        draw_circle(
            pos.x,
            pos.y,
            26.0 * (0.8 + 0.2 * ease),
            Color::new(SUMMON_GOLD.r, SUMMON_GOLD.g, SUMMON_GOLD.b, 0.14 * fade),
        );
        draw_circle(
            pos.x,
            pos.y,
            14.0 * (0.8 + 0.2 * ease),
            Color::new(SUMMON_GOLD.r, SUMMON_GOLD.g, SUMMON_GOLD.b, 0.24 * fade),
        );
        draw_circle(
            pos.x,
            pos.y,
            6.0,
            Color::new(SUMMON_WHITE.r, SUMMON_WHITE.g, SUMMON_WHITE.b, 0.6 * fade),
        );

        // One thin ring expanding outward.
        draw_circle_lines(
            pos.x,
            pos.y,
            10.0 + 34.0 * ease,
            (2.0 * (1.0 - t)).max(0.5),
            Color::new(SUMMON_GOLD.r, SUMMON_GOLD.g, SUMMON_GOLD.b, 0.5 * (1.0 - t)),
        );

        // Gentle drifting sparkles, twinkling as they slow.
        for spark in self.sparks.iter() {
            let twinkle = 0.7 + 0.3 * (self.shield_life * 20.0).sin();
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

    /// The shield's steady-state magical presence: a soft breathing halo
    /// behind the sprite and the motes drifting up off its face. Both are
    /// scaled by the shield's alpha so they fade in and out with it.
    fn draw_life_glow(&self, pos: Vec2, alpha: f32) {
        if alpha <= 0.0 {
            return;
        }
        let pulse = 0.5 + 0.5 * (self.shield_life * 2.5).sin();

        // Soft breathing halo behind the shield.
        draw_circle(
            pos.x,
            pos.y,
            34.0 + 3.0 * pulse,
            Color::new(SUMMON_GOLD.r, SUMMON_GOLD.g, SUMMON_GOLD.b, 0.07 * alpha),
        );
        draw_circle(
            pos.x,
            pos.y,
            26.0 + 2.0 * pulse,
            Color::new(SUMMON_GOLD.r, SUMMON_GOLD.g, SUMMON_GOLD.b, 0.05 * alpha),
        );

        // A thin golden ring that slowly orbits and shimmers, so the shield
        // reads as a spinning ward rather than a static disc.
        let ring_angle = self.shield_life * 1.4;
        let ring_radius = 30.0 + 2.5 * pulse;
        for k in 0..3 {
            let a = ring_angle + k as f32 * std::f32::consts::TAU / 3.0;
            let p = pos + vec2(a.cos(), a.sin()) * ring_radius;
            draw_star_sparkle(
                p,
                4.0 + 2.0 * pulse,
                Color::new(SUMMON_WHITE.r, SUMMON_WHITE.g, SUMMON_WHITE.b, 0.5 * alpha),
            );
        }

        // Rising motes, twinkling and fading as their life drains.
        for mote in self.motes.iter() {
            let life_fade = (mote.life / mote.max_life).clamp(0.0, 1.0);
            let twinkle = 0.6 + 0.4 * (mote.phase + self.shield_life * MOTE_TWINKLE_SPEED).sin();
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
}

impl System for GuardianShieldSystem {
    fn update(&mut self, ctx: &mut Context) {
        let events: Vec<SkillCastEvent> = self.queue.borrow_mut().drain(..).collect();
        for event in events {
            self.begin_cast(&event);
        }

        if self.stance_remaining > 0.0 {
            self.stance_remaining -= ctx.dt;
            if self.stance_remaining <= 0.0 {
                self.shield_active = true;
                self.shield_life = 0.0;
                self.release_lock();
                self.spawn_area_rect();
                if let Some(center) = self.knight_center() {
                    self.spawn_sparks(center);
                }
            }
        }

        if self.shield_active {
            self.shield_life += ctx.dt;
            self.update_area_rect();
            // Sparkles drift, slow, and stop once the flash window ends.
            for spark in self.sparks.iter_mut() {
                spark.pos += spark.vel * ctx.dt;
                spark.vel *= (1.0 - 3.0 * ctx.dt).max(0.0);
            }
            // Motes: spawn at a steady rate, drift upward, and fade out.
            self.mote_timer -= ctx.dt;
            if self.mote_timer <= 0.0 {
                self.mote_timer = 1.0 / MOTES_PER_SEC;
                if let Some(pos) = self.shield_pos() {
                    self.spawn_mote(pos);
                }
            }
            for mote in self.motes.iter_mut() {
                mote.pos += mote.vel * ctx.dt;
                mote.vel.x += (self.shield_life * 8.0 + mote.pos.y * 0.05).sin() * 6.0 * ctx.dt;
                mote.life -= ctx.dt;
            }
            self.motes.retain(|m| m.life > 0.0);
            if self.shield_life >= SHIELD_SECS {
                self.shield_active = false;
                self.shield_life = 0.0;
                self.sparks.clear();
                self.motes.clear();
                self.remove_area_rect();
            }
        }
    }

    fn draw(&self, _ctx: &Context) {
        self.draw_shield();
    }
}

/// Builds the additive-blend material for the glow passes. Pass-through
/// shader; the magic is the pipeline's blend state: dst = src*a + dst, so
/// overlapping glows accumulate toward white-hot instead of muddying.
fn load_glow_material() -> Option<Material> {
    let additive = BlendState::new(
        Equation::Add,
        BlendFactor::Value(BlendValue::SourceAlpha),
        BlendFactor::One,
    );
    let keep_alpha = BlendState::new(Equation::Add, BlendFactor::Zero, BlendFactor::One);

    match load_material(
        ShaderSource::Glsl {
            vertex: ADDITIVE_VERT,
            fragment: ADDITIVE_FRAG,
        },
        MaterialParams {
            pipeline_params: PipelineParams {
                color_blend: Some(additive),
                alpha_blend: Some(keep_alpha),
                ..Default::default()
            },
            ..Default::default()
        },
    ) {
        Ok(material) => Some(material),
        Err(err) => {
            warn!("guardian shield glow material failed to compile: {}", err);
            None
        }
    }
}

/// Builds the golden material for the shield sprite. Reuses the knight's
/// shared `knight_golden_glow` shader (the same one the sword cast uses), so
/// the shield matches the sword's golden magic. Standard alpha blend so the
/// sprite composites over the scene.
fn load_shield_material(assets: &Assets) -> Option<Material> {
    let vertex = assets.shader(shader::Shader::DashFxVert);
    let fragment = assets.shader(shader::Shader::KnightGoldenGlowFrag);

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
            warn!("guardian shield material failed to compile: {}", err);
            None
        }
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
