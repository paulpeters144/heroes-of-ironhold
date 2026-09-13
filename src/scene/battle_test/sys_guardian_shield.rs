use crate::entity::knight::{Facing, Knight, KnightLock, IDLE_FRAME, SWIPE_FRAME};
use crate::entity::player::PlayerOne;
use crate::entity::skills::SkillIconKind;
use crate::events::SkillCastEvent;
use crate::systems::System;
use crate::{images, Animation, Assets, Context, EStore, EventBus, SubCollection};
use macroquad::prelude::*;
use pico_entity_store::entity_ref::EntityRef;
use pico_entity_store::store::IntoChild;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

/// How long the knight is held in the swipe pose before the shield appears
/// (seconds) — the shield appears 250ms after the cast event starts.
const STANCE_SECS: f32 = 0.25;
/// How long the shield stays visible after it appears (seconds).
const SHIELD_SECS: f32 = 15.0;
/// Time (seconds) the shield spends fading in after it appears.
const FADE_IN_SECS: f32 = 0.25;
/// Time (seconds) the shield spends fading out before it despawns.
const FADE_OUT_SECS: f32 = 0.25;
/// How far in front of the knight's center the shield sits (world units).
const SHIELD_AHEAD: f32 = 50.0;
/// The shield's maximum opacity (semi-transparent).
const SHIELD_ALPHA: f32 = 0.25;
/// Guardian shield footprint: matches both the knight's frame (factory_hero
/// FRAME_SIZE, 64) and the PNG's native 64x64, so it draws unscaled. Named
/// GUARDIAN_SHIELD_SIZE, not SHIELD_SIZE, to avoid clashing with
/// factory_hero::SHIELD_SIZE (32.0, the knight's own carried shield).
const GUARDIAN_SHIELD_SIZE: f32 = 64.0;

pub struct GuardianShieldSystem {
    store: Rc<EStore>,
    queue: Rc<RefCell<VecDeque<SkillCastEvent>>>,
    _subs: Rc<SubCollection>,
    stance_remaining: f32,
    shield_active: bool,
    shield_life: f32,
    shield_tex: Texture2D,
}

impl GuardianShieldSystem {
    pub fn new(store: Rc<EStore>, bus: Rc<EventBus>, assets: &Assets) -> Self {
        let queue = Rc::new(RefCell::new(VecDeque::new()));
        let subs = Rc::new(SubCollection::new());
        let queue_for_handler = queue.clone();
        subs.on::<SkillCastEvent>(&bus, move |event: &SkillCastEvent| {
            queue_for_handler.borrow_mut().push_back(event.clone());
        });
        Self {
            store,
            queue,
            _subs: subs,
            stance_remaining: 0.0,
            shield_active: false,
            shield_life: 0.0,
            shield_tex: assets.texture(images::Knight::LgShield),
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
        self.store
            .add(knight, &[KnightLock { by: "GuardianShield" }.into_child()]);

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
    /// plus the current facing direction times SHIELD_AHEAD. Re-read each frame
    /// so the shield tracks the knight and stays in front even if it turns.
    fn shield_position(&self, knight_center: Vec2, facing: Facing) -> Vec2 {
        let dir = match facing {
            Facing::Left => -1.0,
            Facing::Right => 1.0,
        };
        knight_center + vec2(dir * SHIELD_AHEAD, 0.0)
    }

    /// Draws the shield sprite in front of the knight when shield_active is
    /// true: a plain draw_texture_ex of shield_tex at GUARDIAN_SHIELD_SIZE
    /// square (1:1 with the 64x64 PNG), top-left = shield_position -
    /// GUARDIAN_SHIELD_SIZE/2. Position re-resolves the knight's current center
    /// and facing each frame so the shield tracks the knight. The fade is alpha
    /// in the draw color: Color::new(1.0, 1.0, 1.0, alpha) — ramping up over
    /// FADE_IN_SECS at spawn and down over FADE_OUT_SECS before despawn. No
    /// material: macroquad's default pipeline alpha-blends textured draws.
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

        let pos = self.shield_position(center, facing);

        let fade_in = (self.shield_life / FADE_IN_SECS).min(1.0);
        let fade_out = ((self.shield_life - (SHIELD_SECS - FADE_OUT_SECS)) / FADE_OUT_SECS)
            .clamp(0.0, 1.0);
        let alpha = SHIELD_ALPHA * fade_in * (1.0 - fade_out);

        let top_left = pos - vec2(GUARDIAN_SHIELD_SIZE, GUARDIAN_SHIELD_SIZE) * 0.5;
        draw_texture_ex(
            &self.shield_tex,
            top_left.x,
            top_left.y,
            Color::new(1.0, 1.0, 1.0, alpha),
            DrawTextureParams {
                dest_size: Some(vec2(GUARDIAN_SHIELD_SIZE, GUARDIAN_SHIELD_SIZE)),
                flip_x: facing == Facing::Left,
                ..Default::default()
            },
        );
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
            }
        }

        if self.shield_active {
            self.shield_life += ctx.dt;
            if self.shield_life >= SHIELD_SECS {
                self.shield_active = false;
                self.shield_life = 0.0;
            }
        }
    }

    fn draw(&self, _ctx: &Context) {
        self.draw_shield();
    }
}
