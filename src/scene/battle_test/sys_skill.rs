use crate::access::{FontTag, GameFont, TextStyle};
use crate::entity::knight::{Knight, KnightLock, IDLE_FRAME};
use crate::entity::{
    LastUsedSkill, PlayerOne, Skill, SkillDirection, SkillIcon, SkillIconKind, SkillsWidget,
};
use crate::events::{
    SkillActiveEndEvent, SkillActiveEvent, SkillCastEvent, SkillCooldownEvent,
};
use crate::input::{self, Input};
use crate::prelude::*;
use crate::ui::{
    draw_rounded_rect, draw_rounded_rect_lines, draw_skill_slot, SkillIconTextures,
    SKILL_SLOT_SIZE,
};
use crate::util::view_scale;
use crate::{images, Assets, Config};
use macroquad::prelude::*;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

// Prototype placement: selector squares sit this far from the knight's center.
const OFFSET: f32 = 40.0;

// The selector options pop in immediately and scale from 0 to 1 over this long.
const SKILL_SELECTOR_POP: f32 = 0.1;

// The direction highlighted when the selector opens, before any input.
const SKILL_DEFAULT_DIRECTION: SkillDirection = SkillDirection::Up;

// Bar layout (virtual 640x360 screen space): a framed bar of four square slots,
// anchored bottom-center. Slot 0 is the attack slot; the rest are placeholders.
const SLOT: f32 = SKILL_SLOT_SIZE;
const GAP: f32 = 3.2;
const PAD: f32 = 4.0;
const BOTTOM: f32 = 5.0;
const BAR_RADIUS: f32 = 3.2;

// Bar frame palette: dark edge, steel rim, navy back.
const FRAME: Color = Color::new(0.08, 0.09, 0.12, 1.0);
const RIM: Color = Color::new(0.45, 0.48, 0.56, 1.0);
const BACK: Color = Color::new(0.13, 0.13, 0.19, 1.0);

// Key cap: a small brass badge hanging off the slot's bottom edge.
const KEY_W: f32 = 18.0;
const KEY_H: f32 = 15.0;
const KEY_RADIUS: f32 = 3.0;
const KEY_BLEED: f32 = 9.0;

const KEY_INK: Color = Color::new(0.13, 0.1, 0.04, 1.0);
const KEY_CAP: Color = Color::new(0.74, 0.57, 0.23, 1.0);
const KEY_CAP_HI: Color = Color::new(0.9, 0.74, 0.36, 1.0);
const KEY_CAP_EDGE: Color = Color::new(0.4, 0.28, 0.1, 1.0);

// Cooldown overlay: a dark shroud that unwinds clockwise as the skill recovers,
// revealing the icon underneath. Paired with a numeric countdown and a brief
// gold ring flash the instant the skill comes off cooldown.
const COOLDOWN_SHROUD: Color = Color::new(0.03, 0.03, 0.05, 0.62);

const COOLDOWN_INSET: f32 = 3.0;
const COOLDOWN_SWEEP_STEPS: usize = 24;
const READY_FLASH_SECS: f32 = 0.18;
const READY_RING: Color = Color::new(1.0, 0.85, 0.35, 1.0);

// Active-zone state: a persistent area (Consecration, Divine Stance) is "up"
// rather than "recharging", so it gets a steady gold ring + wash instead of the
// dark cooldown sweep, with a countdown of the zone's remaining lifetime.
const ACTIVE_RING: Color = Color::new(1.0, 0.82, 0.3, 1.0);
const ACTIVE_WASH: Color = Color::new(1.0, 0.82, 0.3, 0.16);


/// Remaining/total cooldown for one skill, keyed by its icon kind.
struct SkillCooldown {
    remaining: f32,
    total: f32,
}

/// Remaining zone lifetime for an active zone skill, keyed by its icon kind.
struct SkillActive {
    remaining: f32,
}

/// Draws the dark cooldown shroud as a radial wedge that unwinds clockwise:
/// at `frac = 1.0` the whole circle is covered, and as `frac` drops the wedge
/// recedes clockwise from 12 o'clock until nothing is left. This is the
/// standard "clock wipe" players read instantly.
fn cooldown_sweep(cx: f32, cy: f32, r: f32, frac: f32, color: Color) {
    if frac <= 0.0 {
        return;
    }
    if frac >= 1.0 {
        draw_circle(cx, cy, r, color);
        return;
    }

    // The covered wedge starts where the revealed sweep ended and runs the rest
    // of the way around the clock back to 12 o'clock.
    let start = -std::f32::consts::FRAC_PI_2 + (1.0 - frac) * std::f32::consts::TAU;
    let end = -std::f32::consts::FRAC_PI_2 + std::f32::consts::TAU;

    let steps = ((frac * COOLDOWN_SWEEP_STEPS as f32).ceil() as usize).max(2);
    let mut rim = Vec::with_capacity(steps + 1);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let a = start + (end - start) * t;
        rim.push(vec2(cx + a.cos() * r, cy + a.sin() * r));
    }
    let apex = vec2(cx, cy);
    for i in 0..rim.len() - 1 {
        draw_triangle(apex, rim[i], rim[i + 1], color);
    }
}

pub struct SkillSystem {
    cfg: Rc<Config>,
    store: Rc<EStore>,
    bus: Rc<EventBus>,
    font: GameFont,
    icons: SkillIconTextures,
    movement_gate: Rc<Cell<bool>>,
    selector_open: bool,
    selector_scale: f32,
    pending: Option<SkillDirection>,
    cooldown_queue: Rc<RefCell<VecDeque<SkillCooldownEvent>>>,
    active_queue: Rc<RefCell<VecDeque<SkillActiveEvent>>>,
    active_end_queue: Rc<RefCell<VecDeque<SkillActiveEndEvent>>>,
    _cooldown_subs: Rc<SubCollection>,
    cooldowns: HashMap<SkillIconKind, SkillCooldown>,
    actives: HashMap<SkillIconKind, SkillActive>,
    ready_flash: f32,
}

impl SkillSystem {
    pub fn new(
        cfg: Rc<Config>,
        assets: &Assets,
        store: Rc<EStore>,
        bus: Rc<EventBus>,
        movement_gate: Rc<Cell<bool>>,
    ) -> Self {
        let base = assets.get_font(&TextStyle::new(FontTag::Body));
        let font = GameFont {
            size: 12,
            color: KEY_INK,
            ..base
        };
        let icons = SkillIconTextures {
            slot: assets.texture(images::Ui::SkillSlot),
            blade_barrage: assets.texture(images::SkillIcon::BladeBarrage),
            shield: assets.texture(images::SkillIcon::Shield),
            fireball: assets.texture(images::SkillIcon::Blast),
        };
        let cooldown_queue = Rc::new(RefCell::new(VecDeque::new()));
        let active_queue = Rc::new(RefCell::new(VecDeque::new()));
        let active_end_queue = Rc::new(RefCell::new(VecDeque::new()));
        let cooldown_subs = Rc::new(SubCollection::new());
        let queue_for_handler = cooldown_queue.clone();
        cooldown_subs.on::<SkillCooldownEvent>(&bus, move |event: &SkillCooldownEvent| {
            queue_for_handler.borrow_mut().push_back(event.clone());
        });
        let queue_for_handler = active_queue.clone();
        cooldown_subs.on::<SkillActiveEvent>(&bus, move |event: &SkillActiveEvent| {
            queue_for_handler.borrow_mut().push_back(event.clone());
        });
        let queue_for_handler = active_end_queue.clone();
        cooldown_subs.on::<SkillActiveEndEvent>(&bus, move |event: &SkillActiveEndEvent| {
            queue_for_handler.borrow_mut().push_back(event.clone());
        });
        Self {
            cfg,
            store,
            bus,
            font,
            icons,
            movement_gate,
            selector_open: false,
            selector_scale: 0.0,
            pending: None,
            cooldown_queue,
            active_queue,
            active_end_queue,
            _cooldown_subs: cooldown_subs,
            cooldowns: HashMap::new(),
            actives: HashMap::new(),
            ready_flash: 0.0,
        }
    }

    /// Brass key-cap badge centered on the slot's bottom edge, bleeding below
    /// it so the letter reads as a physical key sticking out of the bar.
    fn key_cap(&self, key: char, slot_x: f32, slot_bottom: f32) {
        let cx = slot_x + SLOT * 0.5;
        let cap_x = cx - KEY_W * 0.5;
        let cap_y = slot_bottom - (KEY_H - KEY_BLEED);

        draw_rounded_rect(cap_x, cap_y, KEY_W, KEY_H, KEY_RADIUS, KEY_CAP_EDGE);
        draw_rounded_rect(
            cap_x + 1.0,
            cap_y + 1.0,
            KEY_W - 2.0,
            KEY_H - 2.0,
            KEY_RADIUS - 1.0,
            KEY_CAP,
        );
        draw_line(
            cap_x + 3.0,
            cap_y + 2.0,
            cap_x + KEY_W - 4.0,
            cap_y + 2.0,
            1.0,
            KEY_CAP_HI,
        );

        let text = key.to_string();
        let scale = view_scale::view_scale(self.cfg.v_width, self.cfg.v_height).0;
        let (font_size, font_scale) = view_scale::crisp_text_params(self.font.size, scale);
        let dims = measure_text(&text, Some(&self.font.font), font_size, font_scale);
        let tx = view_scale::snap_to_pixel(cx - dims.width * 0.5, scale);
        let ty =
            view_scale::snap_to_pixel(cap_y + (KEY_H - dims.height) * 0.5 + dims.offset_y, scale);
        draw_text_ex(
            &text,
            tx,
            ty,
            TextParams {
                font: Some(&self.font.font),
                font_size,
                font_scale,
                color: KEY_INK,
                ..Default::default()
            },
        );
    }

    /// Draws the cooldown treatment over a slot centered at (cx, cy): the
    /// dark radial shroud.
    fn draw_cooldown(&self, cx: f32, cy: f32, cd: &SkillCooldown) {
        let frac = (cd.remaining / cd.total).clamp(0.0, 1.0);
        let r = SLOT * 0.5 - COOLDOWN_INSET;
        cooldown_sweep(cx, cy, r, frac, COOLDOWN_SHROUD);
    }

    /// Draws the "zone active" treatment over a slot: a steady gold ring and
    /// a soft gold wash over the icon. Distinct from the cooldown shroud so
    /// an active area reads as "up now", not "recharging".
    fn draw_active(&self, sx: f32, sy: f32) {
        draw_rounded_rect_lines(
            sx - 3.0,
            sy - 3.0,
            SLOT + 6.0,
            SLOT + 6.0,
            3.0,
            2.0,
            ACTIVE_RING,
        );
        draw_rounded_rect(sx + 2.0, sy + 2.0, SLOT - 4.0, SLOT - 4.0, 2.0, ACTIVE_WASH);
    }

    /// Fire the skill bound to a direction: resolve its icon and the knight's
    /// id, emit a `SkillCastEvent`, and record it as the last used skill.
    fn fire(&mut self, dir: SkillDirection) {
        let kind = self
            .store
            .all::<Skill>()
            .find(|s| s.direction == Some(dir))
            .and_then(|s| self.store.get_child::<SkillIcon>(&s).map(|i| i.kind));

        let caster = self
            .store
            .first::<PlayerOne>()
            .and_then(|p| self.store.get_child::<Knight>(&p))
            .map(|k| k.id());

        if let (Some(caster), Some(kind)) = (caster, kind) {
            self.bus.fire(&SkillCastEvent {
                caster,
                direction: dir,
                kind,
            });
        }

        let last_ref = self
            .store
            .first::<SkillsWidget>()
            .and_then(|w| self.store.get_child::<LastUsedSkill>(&w))
            .map(|l| l.entity_ref());
        if let Some(last_ref) = last_ref {
            self.store.update::<LastUsedSkill, _>(&last_ref, |l| {
                l.icon = kind;
                l.direction = Some(dir);
            });
        }
    }
}

impl System for SkillSystem {
    fn update(&mut self, ctx: &mut Context) {
        // Cooldown bookkeeping runs regardless of selector state: drain the
        // events fired by the skill systems (a frame earlier), start each
        // cooldown, tick them down, and flash when one completes.
        let events: Vec<SkillCooldownEvent> = self.cooldown_queue.borrow_mut().drain(..).collect();
        for event in events {
            self.cooldowns.insert(
                event.kind,
                SkillCooldown {
                    remaining: event.duration,
                    total: event.duration,
                },
            );
        }
        let mut expired: Vec<SkillIconKind> = Vec::new();
        for (kind, cd) in self.cooldowns.iter_mut() {
            cd.remaining = (cd.remaining - ctx.dt).max(0.0);
            if cd.remaining <= 0.0 {
                expired.push(*kind);
            }
        }
        if !expired.is_empty() {
            for kind in expired {
                self.cooldowns.remove(&kind);
            }
            self.ready_flash = READY_FLASH_SECS;
        }
        self.ready_flash = (self.ready_flash - ctx.dt).max(0.0);

        // Active-zone bookkeeping: start a zone on SkillActiveEvent, drop it on
        // SkillActiveEndEvent (flashing "ready"), and tick the countdown down.
        let active_events: Vec<SkillActiveEvent> = self.active_queue.borrow_mut().drain(..).collect();
        for event in active_events {
            self.actives.insert(
                event.kind,
                SkillActive {
                    remaining: event.duration,
                },
            );
        }
        let end_events: Vec<SkillActiveEndEvent> =
            self.active_end_queue.borrow_mut().drain(..).collect();
        for event in end_events {
            self.actives.remove(&event.kind);
            self.ready_flash = READY_FLASH_SECS;
        }
        for active in self.actives.values_mut() {
            active.remaining = (active.remaining - ctx.dt).max(0.0);
        }

        // While a skill holds the knight (e.g. mid-blade-barrage-cast) the selector
        // stays closed and no new cast can be started.
        if self
            .store
            .first::<PlayerOne>()
            .and_then(|p| self.store.get_child::<Knight>(&p))
            .and_then(|k| self.store.get_child::<KnightLock>(&k))
            .is_some()
        {
            return;
        }

        if input::down_once(Input::Skills) {
            self.movement_gate.set(false);
            self.selector_open = true;
            self.selector_scale = 0.0;
            if let Some(anim_ref) = self
                .store
                .first::<PlayerOne>()
                .and_then(|p| self.store.get_child::<Knight>(&p))
                .and_then(|k| self.store.get_child::<Animation>(&k))
                .map(|a| a.entity_ref())
            {
                self.store.update::<Animation, _>(&anim_ref, |a| {
                    a.current_frame = IDLE_FRAME;
                });
            }
            let last_dir = self
                .store
                .first::<SkillsWidget>()
                .and_then(|w| self.store.get_child::<LastUsedSkill>(&w))
                .and_then(|l| l.direction);
            self.pending = Some(last_dir.unwrap_or(SKILL_DEFAULT_DIRECTION));
        }

        if input::down(Input::Skills) && self.selector_open {
            self.selector_scale = (self.selector_scale + ctx.dt / SKILL_SELECTOR_POP).min(1.0);

            if input::down_once(Input::Up) {
                self.pending = Some(SkillDirection::Up);
            }
            if input::down_once(Input::Down) {
                self.pending = Some(SkillDirection::Down);
            }
            if input::down_once(Input::Left) {
                self.pending = Some(SkillDirection::Left);
            }
            if input::down_once(Input::Right) {
                self.pending = Some(SkillDirection::Right);
            }
        }

        if input::up_once(Input::Skills) {
            self.movement_gate.set(true);
            self.selector_open = false;
            self.selector_scale = 0.0;
            let Some(dir) = self.pending.take() else {
                return;
            };
            self.fire(dir);
        }
    }

    fn draw(&self, _ctx: &Context) {
        if !self.selector_open {
            return;
        }

        let Some(center) = self
            .store
            .first::<PlayerOne>()
            .and_then(|p| self.store.get_child::<Knight>(&p))
            .and_then(|k| self.store.get_child::<Animation>(&k))
            .map(|a| a.rect().center())
        else {
            return;
        };

        let offsets = [
            (SkillDirection::Up, vec2(0.0, -OFFSET)),
            (SkillDirection::Down, vec2(0.0, OFFSET)),
            (SkillDirection::Left, vec2(-OFFSET, 0.0)),
            (SkillDirection::Right, vec2(OFFSET, 0.0)),
        ];

        let selected = self.pending.unwrap_or(SKILL_DEFAULT_DIRECTION);
        let scale = self.selector_scale;

        for (dir, off) in offsets {
            let icon = self
                .store
                .all::<Skill>()
                .find(|s| s.direction == Some(dir))
                .and_then(|s| self.store.get_child::<SkillIcon>(&s).map(|i| i.kind));
            let pos = center + off;
            let half = SKILL_SLOT_SIZE * 0.5 * scale;
            draw_skill_slot(
                pos.x - half,
                pos.y - half,
                icon,
                dir == selected,
                &self.icons,
                scale,
            );
        }
    }

    fn draw_ui(&self, _ctx: &Context) {
        let icon = self
            .store
            .first::<SkillsWidget>()
            .and_then(|w| self.store.get_child::<LastUsedSkill>(&w))
            .and_then(|l| l.icon);

        let default_icon = self
            .store
            .all::<Skill>()
            .find(|s| s.direction == Some(SKILL_DEFAULT_DIRECTION))
            .and_then(|s| self.store.get_child::<SkillIcon>(&s).map(|i| i.kind));

        let icon = icon.or(default_icon);

        let n = 4.0;
        let w = PAD * 2.0 + n * SLOT + (n - 1.0) * GAP;
        let h = PAD * 2.0 + SLOT;
        let x = ((self.cfg.v_width - w) * 0.5).round();
        let y = (self.cfg.v_height - BOTTOM - h).round();

        // Bar frame: dark edge, steel rim, navy back.
        draw_rounded_rect(x, y, w, h, BAR_RADIUS, FRAME);
        draw_rounded_rect(x + 1.0, y + 1.0, w - 2.0, h - 2.0, BAR_RADIUS - 1.0, RIM);
        draw_rounded_rect(x + 2.0, y + 2.0, w - 4.0, h - 4.0, BAR_RADIUS - 2.0, BACK);

        for i in 0..4 {
            let sx = x + PAD + i as f32 * (SLOT + GAP);
            let sy = y + PAD;
            let slot_icon: Option<SkillIconKind> = if i == 0 { icon } else { None };
            draw_skill_slot(sx, sy, slot_icon, false, &self.icons, 1.0);
            if i == 0 {
                self.key_cap('a', sx, sy + SLOT);

                if icon.and_then(|kind| self.actives.get(&kind)).is_some() {
                    self.draw_active(sx, sy);
                } else if let Some(cd) = icon.and_then(|kind| self.cooldowns.get(&kind)) {
                    self.draw_cooldown(sx + SLOT * 0.5, sy + SLOT * 0.5, cd);
                }

                if self.ready_flash > 0.0 {
                    let a = (self.ready_flash / READY_FLASH_SECS).clamp(0.0, 1.0);
                    draw_rounded_rect_lines(
                        sx - 3.0,
                        sy - 3.0,
                        SLOT + 6.0,
                        SLOT + 6.0,
                        3.0,
                        2.0,
                        Color::new(READY_RING.r, READY_RING.g, READY_RING.b, a),
                    );
                }
            }
        }
    }
}
