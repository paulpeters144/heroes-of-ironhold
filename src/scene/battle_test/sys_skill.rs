use crate::access::{FontTag, GameFont, TextStyle};
use crate::entity::knight::{Knight, KnightLock, IDLE_FRAME};
use crate::entity::{
    LastUsedSkill, PlayerOne, Skill, SkillDirection, SkillIcon, SkillIconKind, SkillsWidget,
};
use crate::events::SkillCastEvent;
use crate::input::{self, Input};
use crate::prelude::*;
use crate::ui::{draw_rounded_rect, draw_skill_slot, SkillIconTextures, SKILL_SLOT_SIZE};
use crate::util::view_scale;
use crate::{images, Assets, Config};
use macroquad::prelude::*;
use std::cell::Cell;
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
            sword: assets.texture(images::SkillIcon::Swords),
            shield: assets.texture(images::SkillIcon::Shield),
            fireball: assets.texture(images::SkillIcon::Blast),
        };
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
        // While a skill holds the knight (e.g. mid-swords-cast) the selector
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
            }
        }
    }
}
