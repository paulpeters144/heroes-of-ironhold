use crate::entity::SkillIconKind;
use crate::ui::draw_rounded_rect_lines;
use macroquad::prelude::*;

/// Fixed side length of a skill slot, shared by the bar and the selector.
pub const SKILL_SLOT_SIZE: f32 = 28.0;

const SLOT_RADIUS: f32 = 2.0;

// Selection highlight ring.
const HIGHLIGHT: Color = Color::new(1.0, 0.82, 0.2, 1.0);

// Icon palette (procedural fallback glyphs only).
const BROWN: Color = Color::new(0.45, 0.28, 0.15, 1.0);
const POTION_RED: Color = Color::new(0.85, 0.2, 0.22, 1.0);
const GLASS: Color = Color::new(0.65, 0.68, 0.75, 1.0);
const BONE: Color = Color::new(0.85, 0.88, 0.95, 1.0);
const DIM: Color = Color::new(0.4, 0.42, 0.5, 1.0);
const CONSECRATION_GOLD: Color = Color::new(0.95, 0.82, 0.35, 1.0);

/// Textures backing the skill icons, resolved at construction time. Kinds
/// without a texture (Potion, Crossed, empty) fall back to procedural glyphs.
#[derive(Clone)]
pub struct SkillIconTextures {
    pub slot: Texture2D,
    pub blade_barrage: Texture2D,
    pub shield: Texture2D,
    pub fireball: Texture2D,
}

fn skill_texture(kind: SkillIconKind, icons: &SkillIconTextures) -> Option<&Texture2D> {
    match kind {
        SkillIconKind::BladeBarrage => Some(&icons.blade_barrage),
        SkillIconKind::Shield => Some(&icons.shield),
        SkillIconKind::Fireball => Some(&icons.fireball),
        SkillIconKind::ShieldToss => Some(&icons.fireball),
        _ => None,
    }
}

/// Renders one framed slot (frame, well, glyph) with its top-left at (x, y),
/// scaled uniformly around its center by `scale` (1.0 draws at native size).
/// When `highlighted`, a gold ring is drawn around the frame.
pub fn draw_skill_slot(
    x: f32,
    y: f32,
    icon: Option<SkillIconKind>,
    highlighted: bool,
    icons: &SkillIconTextures,
    scale: f32,
) {
    let size = SKILL_SLOT_SIZE * scale;

    if highlighted {
        draw_rounded_rect_lines(
            x - 3.0 * scale,
            y - 3.0 * scale,
            size + 6.0 * scale,
            size + 6.0 * scale,
            (SLOT_RADIUS + 1.0) * scale,
            2.0 * scale,
            HIGHLIGHT,
        );
    }

    draw_texture_ex(
        &icons.slot,
        x - scale,
        y - scale,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(size + 2.0 * scale, size + 2.0 * scale)),
            ..Default::default()
        },
    );

    let cx = x + size * 0.5;
    let cy = y + size * 0.5;
    draw_skill_icon(icon, cx, cy, (SKILL_SLOT_SIZE - 10.0) * scale, icons);
}

/// Draws the icon for a kind ("+" placeholder when `None`), preferring the
/// backing texture and falling back to a procedural glyph.
pub fn draw_skill_icon(
    kind: Option<SkillIconKind>,
    cx: f32,
    cy: f32,
    s: f32,
    icons: &SkillIconTextures,
) {
    let texture = kind.and_then(|k| skill_texture(k, icons));
    match (kind, texture) {
        (_, Some(tex)) => draw_texture_ex(
            tex,
            cx - s * 0.5,
            cy - s * 0.5,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(s, s)),
                ..Default::default()
            },
        ),
        (Some(SkillIconKind::Potion), None) => potion_icon(cx, cy, s),
        (Some(SkillIconKind::Crossed), None) => crossed_icon(cx, cy, s),
        (Some(SkillIconKind::DivineStance), None) => divine_stance_icon(cx, cy, s),
        (Some(SkillIconKind::Consecration), None) => consecration_icon(cx, cy, s),
        _ => plus_icon(cx, cy, s),
    }
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

/// Two crossed strokes (blades/axes placeholder).
fn crossed_icon(cx: f32, cy: f32, s: f32) {
    let r = s * 0.34;
    draw_line(cx - r, cy - r, cx + r, cy + r, 3.0, BONE);
    draw_line(cx - r, cy + r, cx + r, cy - r, 3.0, BONE);
}

/// Two plus signs — one large, one small — offset diagonally, evoking
/// healing. Matches the reference glyph: big cross bottom-left, small cross
/// top-right, both solid and cream-gold.
fn divine_stance_icon(cx: f32, cy: f32, s: f32) {
    let color = Color::new(0.95, 0.90, 0.55, 1.0);

    // Large plus at bottom-left — 13x13 with 4px thick arms
    let big_len = (s * 0.44).round();
    let big_thick = (s * 0.14).round();
    let bx = (cx - 5.0).round();
    let by = (cy + 5.0).round();
    let bhl = (big_len * 0.5).round();
    let bht = (big_thick * 0.5).round();
    draw_rectangle(bx - bhl, by - bht, big_len, big_thick, color);
    draw_rectangle(bx - bht, by - bhl, big_thick, big_len, color);

    // Small plus at top-right — exactly 7x7 with 3px thick arms, integer coords
    let sx = (cx + 5.0).round();
    let sy = (cy - 5.0).round();
    draw_rectangle(sx - 3.0, sy - 1.0, 7.0, 3.0, color);
    draw_rectangle(sx - 1.0, sy - 3.0, 3.0, 7.0, color);
}

/// Gold shield with a centered cross, for the armor-blessing zone.
fn consecration_icon(cx: f32, cy: f32, s: f32) {
    let w = s * 0.32;
    let h = s * 0.38;
    let shield = [
        vec2(cx, cy - h),
        vec2(cx + w, cy - h * 0.6),
        vec2(cx + w, cy + h * 0.2),
        vec2(cx, cy + h),
        vec2(cx - w, cy + h * 0.2),
        vec2(cx - w, cy - h * 0.6),
    ];
    for i in 0..6 {
        let a = shield[i];
        let b = shield[(i + 1) % 6];
        draw_line(a.x, a.y, b.x, b.y, 2.0, CONSECRATION_GOLD);
    }
    let cross_len = s * 0.18;
    let cross_thick = 2.0;
    draw_rectangle(cx - cross_len * 0.5, cy - cross_thick * 0.5, cross_len, cross_thick, CONSECRATION_GOLD);
    draw_rectangle(cx - cross_thick * 0.5, cy - cross_len * 0.5, cross_thick, cross_len, CONSECRATION_GOLD);
}

/// Dim "+" for an empty slot.
fn plus_icon(cx: f32, cy: f32, s: f32) {
    let len = s * 0.44;
    let t = 3.0;
    draw_rectangle(cx - len * 0.5, cy - t * 0.5, len, t, DIM);
    draw_rectangle(cx - t * 0.5, cy - len * 0.5, t, len, DIM);
}
