use crate::entity::skills::{Skill, SkillIcon, SkillIconKind, SkillsWidget};
use crate::ui::draw_rounded_rect;
use crate::util::view_scale;
use crate::{Assets, Config, Context, EStore, FontTag, GameFont, TextStyle};
use macroquad::prelude::*;
use std::rc::Rc;

// Layout (virtual 640x360 screen space): a framed bar of square slots,
// anchored bottom-center regardless of slot count.
const SLOT: f32 = 28.0;
const GAP: f32 = 3.2;
const PAD: f32 = 4.0;
const BOTTOM: f32 = 5.0;
const BAR_RADIUS: f32 = 3.2;
const SLOT_RADIUS: f32 = 2.0;

// Palette: dark steel frame, warm steel rim, deep navy wells.
const FRAME: Color = Color::new(0.08, 0.09, 0.12, 1.0);
const RIM: Color = Color::new(0.45, 0.48, 0.56, 1.0);
const BACK: Color = Color::new(0.13, 0.13, 0.19, 1.0);
const WELL: Color = Color::new(0.17, 0.17, 0.25, 1.0);
const WELL_EDGE: Color = Color::new(0.35, 0.37, 0.46, 1.0);
const WELL_HI: Color = Color::new(0.27, 0.28, 0.37, 1.0);
const WELL_LOW: Color = Color::new(0.11, 0.11, 0.17, 1.0);
const SELECT: Color = Color::new(0.95, 0.45, 0.13, 1.0);
const SELECT_GLOW: Color = Color::new(0.95, 0.45, 0.13, 0.3);
const SELECT_TINT: Color = Color::new(0.95, 0.45, 0.13, 0.14);

// Icon palette.
const SILVER: Color = Color::new(0.8, 0.82, 0.88, 1.0);
const GOLD: Color = Color::new(0.8, 0.62, 0.24, 1.0);
const BROWN: Color = Color::new(0.45, 0.28, 0.15, 1.0);
const BRONZE: Color = Color::new(0.6, 0.42, 0.24, 1.0);
const BRONZE_LIGHT: Color = Color::new(0.76, 0.58, 0.35, 1.0);
const POTION_RED: Color = Color::new(0.85, 0.2, 0.22, 1.0);
const GLASS: Color = Color::new(0.65, 0.68, 0.75, 1.0);
const FIRE_ORANGE: Color = Color::new(0.95, 0.45, 0.1, 1.0);
const FIRE_RED: Color = Color::new(0.85, 0.22, 0.08, 1.0);
const FIRE_YELLOW: Color = Color::new(1.0, 0.8, 0.25, 1.0);
const BONE: Color = Color::new(0.85, 0.88, 0.95, 1.0);
const DIM: Color = Color::new(0.4, 0.42, 0.5, 1.0);
// Key cap: a small brass badge hanging off the slot's bottom edge.
const KEY_W: f32 = 18.0;
const KEY_H: f32 = 15.0;
const KEY_RADIUS: f32 = 3.0;
const KEY_BLEED: f32 = 9.0;

const KEY_INK: Color = Color::new(0.13, 0.1, 0.04, 1.0);
const KEY_CAP: Color = Color::new(0.74, 0.57, 0.23, 1.0);
const KEY_CAP_HI: Color = Color::new(0.9, 0.74, 0.36, 1.0);
const KEY_CAP_EDGE: Color = Color::new(0.4, 0.28, 0.1, 1.0);

/// Snapshot of one slot, copied out of the store before drawing.
struct SlotView {
    icon: Option<SkillIconKind>,
    selected: bool,
    key: Option<char>,
}

pub struct SkillsBarDrawSystem {
    cfg: Rc<Config>,
    store: Rc<EStore>,
    font: GameFont,
}

impl SkillsBarDrawSystem {
    pub fn new(cfg: Rc<Config>, assets: &Assets, store: Rc<EStore>) -> Self {
        let base = assets.get_font(&TextStyle::new(FontTag::Body));
        let font = GameFont {
            size: 12,
            color: KEY_INK,
            ..base
        };
        Self { cfg, store, font }
    }

    /// Reads `SkillsWidget -> Skill -> SkillIcon` in bar order.
    fn slots(&self) -> Vec<SlotView> {
        let child_refs = {
            let Some(widget) = self.store.first::<SkillsWidget>() else {
                return Vec::new();
            };
            self.store.children(&widget)
        };
        child_refs
            .iter()
            .filter_map(|eref| {
                let skill = self.store.get_by_id::<Skill>(eref.id())?;
                let icon = self.store.get_child::<SkillIcon>(&skill).map(|i| i.kind);
                Some(SlotView {
                    icon,
                    selected: skill.selected,
                    key: skill.key,
                })
            })
            .collect()
    }

    pub fn draw(&self, _ctx: &Context) {
        let slots = self.slots();
        if slots.is_empty() {
            return;
        }

        let n = slots.len() as f32;
        let w = PAD * 2.0 + n * SLOT + (n - 1.0) * GAP;
        let h = PAD * 2.0 + SLOT;
        let x = ((self.cfg.v_width - w) * 0.5).round();
        let y = (self.cfg.v_height - BOTTOM - h).round();

        // Bar frame: dark edge, steel rim, navy back.
        draw_rounded_rect(x, y, w, h, BAR_RADIUS, FRAME);
        draw_rounded_rect(x + 1.0, y + 1.0, w - 2.0, h - 2.0, BAR_RADIUS - 1.0, RIM);
        draw_rounded_rect(x + 2.0, y + 2.0, w - 4.0, h - 4.0, BAR_RADIUS - 2.0, BACK);

        for (i, slot) in slots.iter().enumerate() {
            let sx = x + PAD + i as f32 * (SLOT + GAP);
            let sy = y + PAD;
            self.slot(sx, sy, slot);
        }
    }

    /// One rounded well: dark separation gap, steel edge, navy fill, top
    /// highlight and bottom shade, an optional select tint, then the icon
    /// (or a dim "+" when empty), select glow, and the key cap on top.
    fn slot(&self, x: f32, y: f32, slot: &SlotView) {
        draw_rectangle(x - 1.0, y - 1.0, SLOT + 2.0, SLOT + 2.0, FRAME);
        draw_rounded_rect(x, y, SLOT, SLOT, SLOT_RADIUS, WELL_EDGE);
        draw_rounded_rect(
            x + 1.0,
            y + 1.0,
            SLOT - 2.0,
            SLOT - 2.0,
            SLOT_RADIUS - 1.0,
            WELL,
        );
        draw_line(x + 3.0, y + 2.0, x + SLOT - 4.0, y + 2.0, 1.0, WELL_HI);
        draw_line(
            x + 2.0,
            y + SLOT - 2.0,
            x + SLOT - 3.0,
            y + SLOT - 2.0,
            1.0,
            WELL_LOW,
        );

        if slot.selected {
            draw_rounded_rect(
                x + 1.0,
                y + 1.0,
                SLOT - 2.0,
                SLOT - 2.0,
                SLOT_RADIUS - 1.0,
                SELECT_TINT,
            );
        }

        let cx = x + SLOT * 0.5;
        let cy = y + SLOT * 0.5;
        match slot.icon {
            Some(kind) => Self::icon(kind, cx, cy, SLOT - 10.0),
            None => Self::plus_icon(cx, cy, SLOT - 10.0),
        }

        if slot.selected {
            draw_rectangle_lines(x - 1.0, y - 1.0, SLOT + 2.0, SLOT + 2.0, 2.0, SELECT);
            draw_rectangle_lines(x - 2.0, y - 2.0, SLOT + 4.0, SLOT + 4.0, 1.0, SELECT_GLOW);
        }

        if let Some(key) = slot.key {
            self.key_cap(key, x, y + SLOT);
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

    fn icon(kind: SkillIconKind, cx: f32, cy: f32, s: f32) {
        match kind {
            SkillIconKind::Sword => Self::sword_icon(cx, cy, s),
            SkillIconKind::Shield => Self::shield_icon(cx, cy, s),
            SkillIconKind::Potion => Self::potion_icon(cx, cy, s),
            SkillIconKind::Fireball => Self::fireball_icon(cx, cy, s),
            SkillIconKind::Crossed => Self::crossed_icon(cx, cy, s),
        }
    }

    /// Diagonal blade pointing up-right, gold guard, brown hilt.
    fn sword_icon(cx: f32, cy: f32, s: f32) {
        let d = vec2(1.0, -1.0).normalize();
        let p = vec2(1.0, 1.0).normalize();
        let center = vec2(cx, cy);
        let tip = center + d * s * 0.45;
        let guard = center - d * s * 0.12;
        let hilt = center - d * s * 0.4;

        let blade_base = tip - d * s * 0.14;
        draw_line(guard.x, guard.y, blade_base.x, blade_base.y, 3.0, SILVER);
        draw_triangle(tip, blade_base + p * 2.0, blade_base - p * 2.0, SILVER);

        let g0 = guard - p * s * 0.16;
        let g1 = guard + p * s * 0.16;
        draw_line(g0.x, g0.y, g1.x, g1.y, 2.5, GOLD);
        draw_line(guard.x, guard.y, hilt.x, hilt.y, 2.5, BROWN);
        draw_circle(hilt.x, hilt.y, 1.8, GOLD);
    }

    /// Kite shield: rectangular top, pointed bottom, lighter inset, boss.
    fn shield_icon(cx: f32, cy: f32, s: f32) {
        let w = s * 0.66;
        let h = s * 0.76;
        let x = cx - w * 0.5;
        let y = cy - h * 0.5;
        let mid = y + h * 0.55;

        draw_rectangle(x, y, w, h * 0.55, BRONZE);
        draw_triangle(vec2(x, mid), vec2(x + w, mid), vec2(cx, y + h), BRONZE);

        let ix = x + 2.5;
        let iw = w - 5.0;
        draw_rectangle(ix, y + 2.0, iw, h * 0.55 - 2.5, BRONZE_LIGHT);
        draw_triangle(
            vec2(ix, mid - 0.5),
            vec2(ix + iw, mid - 0.5),
            vec2(cx, y + h - 3.0),
            BRONZE_LIGHT,
        );

        draw_circle(cx, y + h * 0.3, 2.5, SILVER);
    }

    /// Round red flask with a glass neck and cork.
    fn potion_icon(cx: f32, cy: f32, s: f32) {
        draw_rectangle(cx - 2.0, cy - s * 0.34, 4.0, s * 0.22, GLASS);
        draw_rectangle(cx - 2.5, cy - s * 0.42, 5.0, 3.0, BROWN);
        draw_circle(cx, cy + s * 0.1, s * 0.27, POTION_RED);
        draw_circle(
            cx - s * 0.09,
            cy + s * 0.02,
            s * 0.07,
            Color::new(1.0, 1.0, 1.0, 0.35),
        );
    }

    /// Fireball streaking up-right: red/orange tail, yellow-hot head.
    fn fireball_icon(cx: f32, cy: f32, s: f32) {
        let d = vec2(1.0, -1.0).normalize();
        let p = vec2(1.0, 1.0).normalize();
        let head = vec2(cx, cy) + d * s * 0.16;

        for (dist, half, color) in [
            (s * 0.46, 1.5, FIRE_RED),
            (s * 0.33, 2.5, FIRE_ORANGE),
            (s * 0.2, 3.5, FIRE_ORANGE),
        ] {
            let base = head - d * dist;
            draw_triangle(head, base + p * half, base - p * half, color);
        }

        draw_circle(head.x, head.y, s * 0.18, FIRE_ORANGE);
        draw_circle(head.x, head.y, s * 0.1, FIRE_YELLOW);
    }

    /// Two crossed strokes (blades/axes placeholder).
    fn crossed_icon(cx: f32, cy: f32, s: f32) {
        let r = s * 0.34;
        draw_line(cx - r, cy - r, cx + r, cy + r, 3.0, BONE);
        draw_line(cx - r, cy + r, cx + r, cy - r, 3.0, BONE);
    }

    /// Dim "+" for an empty slot.
    fn plus_icon(cx: f32, cy: f32, s: f32) {
        let len = s * 0.44;
        let t = 3.0;
        draw_rectangle(cx - len * 0.5, cy - t * 0.5, len, t, DIM);
        draw_rectangle(cx - t * 0.5, cy - len * 0.5, t, len, DIM);
    }
}
