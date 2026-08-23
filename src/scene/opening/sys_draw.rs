use crate::scene::opening::components::Animation;
use crate::scene::opening::saves::SAVE_SLOTS;
use crate::scene::opening::sys_menu::{MenuState, Phase, EXPAND_DUR, FLASH_DUR, HOLD_DUR, SELECT_DUR};
use crate::systems::Draw;
use crate::ui::{draw_rounded_rect, draw_rounded_rect_lines};
use crate::{font, Assets, Config, Context, EStore, FontTag, GameFont, TextStyle};
use macroquad::prelude::*;

const TITLE: &str = "MEGABOT";

const PANEL_W: f32 = 400.0;
const PANEL_H: f32 = 232.0;
const ROW_H: f32 = 42.0;
const ROW_GAP: f32 = 6.0;
const HEADER_H: f32 = 30.0;

fn bg_color() -> Color {
    Color::new(0.05, 0.05, 0.2, 1.0)
}

fn panel_fill() -> Color {
    Color::new(0.07, 0.08, 0.16, 0.96)
}

fn panel_border() -> Color {
    Color::new(0.30, 0.55, 0.85, 1.0)
}

fn row_fill() -> Color {
    Color::new(0.10, 0.12, 0.24, 1.0)
}

fn row_fill_focused() -> Color {
    Color::new(0.16, 0.22, 0.42, 1.0)
}

fn row_border_focused() -> Color {
    Color::new(0.55, 0.85, 1.0, 1.0)
}

fn accent_color() -> Color {
    Color::new(0.40, 0.80, 1.0, 1.0)
}

pub struct DrawSysParms {
    pub store: &'static EStore,
    pub cfg: &'static Config,
    pub assets: &'static Assets,
}

pub struct DrawSys {
    store: &'static EStore,
    cfg: &'static Config,
    assets: &'static Assets,
    body_font: Option<GameFont>,
    h1_font: Option<GameFont>,
    h3_font: Option<GameFont>,
    tiny_font: Option<GameFont>,
}

impl DrawSys {
    pub fn new(parms: DrawSysParms) -> Self {
        let body_font = parms.assets.get_font(&TextStyle::new(FontTag::Body));
        let h1_font = parms.assets.get_font(&TextStyle::new(FontTag::H1));
        let h3_font = parms.assets.get_font(&TextStyle::new(FontTag::H3));
        let tiny_font = parms.assets.get_font(&TextStyle::new(FontTag::Tiny));
        Self {
            store: parms.store,
            cfg: parms.cfg,
            assets: parms.assets,
            body_font,
            h1_font,
            h3_font,
            tiny_font,
        }
    }

    fn font(&self) -> Option<&Font> {
        self.assets.font(font::Id::Pixellari)
    }

    fn draw_text(
        &self,
        text: &str,
        x: f32,
        y: f32,
        size: f32,
        color: Color,
        font: Option<&Font>,
    ) {
        draw_text_ex(
            text,
            x,
            y,
            TextParams {
                font,
                font_size: size as u16,
                font_scale: 1.0,
                color,
                ..Default::default()
            },
        );
    }

    fn draw_text_centered(
        &self,
        text: &str,
        cx: f32,
        cy: f32,
        size: f32,
        color: Color,
    ) {
        let dims = measure_text(text, self.font(), size as u16, 1.0);
        let x = cx - dims.width * 0.5;
        let y = cy - dims.height * 0.5 + dims.offset_y;
        self.draw_text(text, x, y, size, color, self.font());
    }

    fn draw_text_centered_font(
        &self,
        text: &str,
        cx: f32,
        cy: f32,
        gf: &GameFont,
        color: Color,
    ) {
        let dims = measure_text(text, Some(gf.font), gf.size, 1.0);
        let x = cx - dims.width * 0.5;
        let y = cy - dims.height * 0.5 + dims.offset_y;
        self.draw_text(text, x, y, gf.size as f32, color, Some(gf.font));
    }

    fn draw_text_right(
        &self,
        text: &str,
        right_x: f32,
        y: f32,
        gf: &GameFont,
        color: Color,
    ) {
        let dims = measure_text(text, Some(gf.font), gf.size, 1.0);
        self.draw_text(text, right_x - dims.width, y, gf.size as f32, color, Some(gf.font));
    }

    // ── Attract phase ────────────────────────────────────────────────

    fn draw_background(&self) {
        draw_rectangle(0.0, 0.0, self.cfg.v_width, self.cfg.v_height, bg_color());
    }

    fn draw_mb_running(&self) {
        let Some(anim) = self.store.first::<Animation>() else {
            return;
        };
        let frame_w = anim.texture.width() / anim.frame_count as f32;
        let frame_h = anim.texture.height();
        let source = Rect::new(anim.frame as f32 * frame_w, 0.0, frame_w, frame_h);
        let w = frame_w * anim.scale;
        let h = frame_h * anim.scale;
        draw_texture_ex(
            &anim.texture,
            anim.pos.x - w * 0.5,
            anim.pos.y - h * 0.5,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(w, h)),
                source: Some(source),
                ..Default::default()
            },
        );
    }

    fn draw_press_enter(&self) {
        let Some(anim) = self.store.first::<Animation>() else {
            return;
        };
        let Some(state) = self.store.first::<MenuState>() else {
            return;
        };
        if state.phase != Phase::Attract {
            return;
        }
        let frame_h = anim.texture.height();
        let bottom = anim.pos.y + frame_h * 0.5;
        let cy = bottom + 48.0;
        let cx = self.cfg.v_width * 0.5;
        if let Some(ref gf) = self.body_font {
            self.draw_text_centered_font("Press Enter...", cx, cy, gf, gf.color);
        }
    }

    fn draw_title(&self, t: f32) {
        let Some(ref gf) = self.h1_font else {
            return;
        };
        let Some(anim) = self.store.first::<Animation>() else {
            return;
        };
        let cx = self.cfg.v_width * 0.5;
        let base = gf.size as f32;
        let target = self.cfg.v_width * 0.55;
        let measured = measure_text(TITLE, self.font(), gf.size, 1.0);
        let size = (base * (target / measured.width)).round();
        let dims = measure_text(TITLE, self.font(), size as u16, 1.0);
        // Position title above the megabot sprite
        let frame_h = anim.texture.height();
        let sprite_top = anim.pos.y - frame_h * 0.5;
        let cy = (sprite_top - dims.height * 0.5 - 44.0 + (t * 1.6).sin() * 1.5).round();

        // shadow
        self.draw_text_centered(TITLE, cx + 2.0, cy + 3.0, size, Color::new(0.0, 0.0, 0.0, 0.85));
        // outline
        for (dx, dy) in [
            (1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0),
            (1.0, 1.0), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0),
        ] {
            self.draw_text_centered(TITLE, cx + dx, cy + dy, size, Color::new(0.02, 0.05, 0.12, 1.0));
        }
        // mid layer
        self.draw_text_centered(TITLE, cx, cy + 1.0, size, Color::new(0.20, 0.45, 0.70, 1.0));
        // top layer
        self.draw_text_centered(TITLE, cx, cy, size, Color::new(0.93, 0.98, 1.0, 1.0));
    }

    // ── Transition phase ─────────────────────────────────────────────

    fn draw_transition(&self, state: &MenuState) {
        if state.phase != Phase::Transition {
            return;
        }
        let t = state.elapsed;
        let cx = self.cfg.v_width * 0.5;
        let cy = self.cfg.v_height * 0.5;

        // 1. white flash — brief full-screen white
        if t < FLASH_DUR {
            let ft = t / FLASH_DUR;
            let alpha = 1.0 - MenuState::ease_out_cubic(ft);
            draw_rectangle(
                0.0,
                0.0,
                self.cfg.v_width,
                self.cfg.v_height,
                Color::new(1.0, 1.0, 1.0, alpha),
            );
            return;
        }

        // 2. panel fade-in — white outline expands then crossfades to dark panel
        let et = ((t - FLASH_DUR) / EXPAND_DUR).clamp(0.0, 1.0);
        if et < 1.0 {
            let e = MenuState::ease_out_cubic(et);
            // expanding white outline
            let w = e * PANEL_W;
            let h = e * PANEL_H;
            draw_rounded_rect_lines(
                cx - w * 0.5,
                cy - h * 0.5,
                w,
                h,
                10.0,
                2.0,
                Color::new(0.95, 0.98, 1.0, 1.0 - e * 0.3),
            );
            // dark fill fading in
            draw_rounded_rect(
                cx - w * 0.5,
                cy - h * 0.5,
                w,
                h,
                10.0,
                Color::new(0.07, 0.08, 0.16, e * 0.96),
            );
            return;
        }

        // 3. hold — dark panel fully formed, brief pause before menu items appear
        let ht = ((t - FLASH_DUR - EXPAND_DUR) / HOLD_DUR).clamp(0.0, 1.0);
        let border_alpha = 1.0 - ht * 0.5;
        draw_rounded_rect(
            cx - PANEL_W * 0.5,
            cy - PANEL_H * 0.5,
            PANEL_W,
            PANEL_H,
            10.0,
            panel_fill(),
        );
        draw_rounded_rect_lines(
            cx - PANEL_W * 0.5,
            cy - PANEL_H * 0.5,
            PANEL_W,
            PANEL_H,
            10.0,
            2.0,
            Color::new(0.95, 0.98, 1.0, border_alpha),
        );
    }

    // ── Menu phase ───────────────────────────────────────────────────

    fn panel_origin(&self) -> (f32, f32) {
        let cx = self.cfg.v_width * 0.5;
        let cy = self.cfg.v_height * 0.52;
        (cx - PANEL_W * 0.5, cy - PANEL_H * 0.5)
    }

    fn draw_menu(&self, state: &MenuState) {
        if state.phase != Phase::Menu && state.phase != Phase::Selected {
            return;
        }
        let (px, py) = self.panel_origin();

        // dim backdrop
        draw_rectangle(
            0.0,
            0.0,
            self.cfg.v_width,
            self.cfg.v_height,
            Color::new(0.0, 0.0, 0.05, 0.55),
        );

        // panel with rounded border
        draw_rounded_rect(px, py, PANEL_W, PANEL_H, 10.0, panel_fill());
        draw_rounded_rect_lines(px, py, PANEL_W, PANEL_H, 10.0, 2.0, panel_border());
        // inner accent line
        draw_rounded_rect_lines(
            px + 3.0,
            py + 3.0,
            PANEL_W - 6.0,
            PANEL_H - 6.0,
            7.0,
            1.0,
            Color::new(0.4, 0.7, 1.0, 0.25),
        );

        // header
        let header_cy = py + HEADER_H * 0.5;
        if let Some(ref gf) = self.h3_font {
            let header_text = if SAVE_SLOTS.is_empty() {
                "NEW GAME"
            } else {
                "SELECT SAVE FILE"
            };
            self.draw_text_centered_font(header_text, px + PANEL_W * 0.5, header_cy, gf, accent_color());
        }
        // header underline
        let uy = py + HEADER_H - 4.0;
        draw_rectangle(px + 14.0, uy, PANEL_W - 28.0, 1.0, Color::new(0.4, 0.7, 1.0, 0.5));

        // menu rows: NEW GAME + 3 save slots
        let items = 1 + SAVE_SLOTS.len();
        for i in 0..items {
            let t = state.item_t(i);
            if t <= 0.0 {
                continue;
            }
            let e = MenuState::ease_out_back(t);
            let row_y = py + HEADER_H + 6.0 + i as f32 * (ROW_H + ROW_GAP);
            let slide = (1.0 - e) * 24.0;
            let row_x = px + 14.0 + slide;
            let row_w = PANEL_W - 28.0;

            let focused = state.focus == i && state.phase == Phase::Menu;
            let selected = state.focus == i && state.phase == Phase::Selected;

            self.draw_menu_row(state, i, row_x, row_y, row_w, ROW_H, t, focused, selected);
        }

        // footer hint (only show when there are save slots to navigate)
        if state.phase == Phase::Menu && !SAVE_SLOTS.is_empty() {
            if let Some(ref gf) = self.tiny_font {
                let fy = py + PANEL_H - 10.0;
                let hint = "UP/DOWN: Navigate    ENTER: Select";
                let dims = measure_text(hint, Some(gf.font), gf.size, 1.0);
                self.draw_text(
                    hint,
                    px + PANEL_W * 0.5 - dims.width * 0.5,
                    fy,
                    gf.size as f32,
                    Color::new(0.55, 0.65, 0.85, 0.8),
                    Some(gf.font),
                );
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_menu_row(
        &self,
        state: &MenuState,
        index: usize,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        t: f32,
        focused: bool,
        selected: bool,
    ) {
        let alpha = t;

        // row background
        let fill = if selected {
            Color::new(0.35, 0.55, 0.85, alpha)
        } else if focused {
            row_fill_focused()
        } else {
            row_fill()
        };
        draw_rounded_rect(x, y, w, h, 6.0, fill);

        // focused border with pulse
        if focused {
            let solo = SAVE_SLOTS.is_empty();
            let pulse = if solo {
                1.0
            } else {
                0.7 + 0.3 * (get_time() as f32 * 6.0).sin()
            };
            let mut border = row_border_focused();
            border.a = pulse * alpha;
            draw_rounded_rect_lines(x, y, w, h, 6.0, 2.0, border);
            // caret arrows only when there is something to navigate to
            if !solo {
                let caret_color = accent_color();
                self.draw_caret(x + 6.0, y + h * 0.5, true, caret_color);
                self.draw_caret(x + w - 6.0, y + h * 0.5, false, caret_color);
            }
        }

        // selected flash overlay
        if selected {
            let st = (state.elapsed / SELECT_DUR).clamp(0.0, 1.0);
            let flash = (1.0 - st) * 0.7;
            draw_rounded_rect(x, y, w, h, 6.0, Color::new(1.0, 1.0, 1.0, flash));
            // expanding outline
            let expand = st * 8.0;
            draw_rounded_rect_lines(
                x - expand * 0.5,
                y - expand * 0.5,
                w + expand,
                h + expand,
                6.0,
                2.0,
                Color::new(1.0, 1.0, 1.0, (1.0 - st) * 0.9),
            );
        }

        let text_alpha = alpha;
        if index == 0 {
            // NEW GAME row
            if let Some(ref gf) = self.h3_font {
                let mut c = gf.color;
                c.a = text_alpha;
                let label = if selected && state.phase == Phase::Selected {
                    ">> STARTING NEW GAME <<"
                } else if SAVE_SLOTS.is_empty() {
                    "START NEW GAME"
                } else {
                    "+ NEW GAME"
                };
                self.draw_text_centered_font(label, x + w * 0.5, y + h * 0.5, gf, c);
            }
        } else {
            let slot = &SAVE_SLOTS[index - 1];
            if selected && state.phase == Phase::Selected {
                // show loading label centered, hide details
                if let Some(ref gf) = self.h3_font {
                    let c = Color::new(1.0, 1.0, 1.0, text_alpha);
                    self.draw_text_centered_font(
                        ">> LOADING SAVE <<",
                        x + w * 0.5,
                        y + h * 0.5,
                        gf,
                        c,
                    );
                }
            } else {
                self.draw_save_slot_row(slot, x, y, w, h, text_alpha);
            }
        }
    }

    fn draw_save_slot_row(
        &self,
        slot: &crate::scene::opening::saves::SaveSlot,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        alpha: f32,
    ) {
        let pad = 22.0;
        let tx = x + pad;
        let ty = y + 8.0;

        // slot name + level
        if let Some(ref gf) = self.h3_font {
            let mut c = gf.color;
            c.a = alpha;
            let left = format!("{}  Lv.{}", slot.name, slot.level);
            self.draw_text(&left, tx, ty + 6.0, gf.size as f32, c, Some(gf.font));

            // chapter on the right
            let mut dim = gf.color;
            dim.a = alpha * 0.75;
            self.draw_text_right(slot.chapter, x + w - pad, ty + 6.0, gf, dim);
        }

        // play time + timestamp (tiny)
        if let Some(ref gf) = self.tiny_font {
            let mut c = Color::new(0.65, 0.75, 0.95, alpha * 0.9);
            self.draw_text(slot.play_time, tx, y + h - 12.0, gf.size as f32, c, Some(gf.font));
            c.a = alpha * 0.6;
            self.draw_text_right(slot.timestamp, x + w - pad, y + h - 12.0, gf, c);
        }

        // progress bar
        let bar_w = w - pad * 2.0 - 60.0;
        let bar_h = 3.0;
        let bar_x = tx;
        let bar_y = y + h - 7.0;
        draw_rounded_rect(bar_x, bar_y, bar_w, bar_h, 1.5, Color::new(0.15, 0.18, 0.3, alpha));
        draw_rounded_rect(
            bar_x,
            bar_y,
            bar_w * slot.progress,
            bar_h,
            1.5,
            Color::new(0.4, 0.85, 0.6, alpha),
        );
        // progress percentage
        if let Some(ref gf) = self.tiny_font {
            let pct = format!("{}%", (slot.progress * 100.0) as u32);
            self.draw_text_right(
                &pct,
                x + w - pad,
                bar_y + 2.0,
                gf,
                Color::new(0.5, 0.9, 0.65, alpha * 0.9),
            );
        }
    }

    fn draw_caret(&self, cx: f32, cy: f32, left: bool, color: Color) {
        let s = 4.0;
        let dir = if left { 1.0 } else { -1.0 };
        draw_triangle(
            vec2(cx + s * dir, cy),
            vec2(cx - s * dir, cy - s),
            vec2(cx - s * dir, cy + s),
            color,
        );
    }
}

impl Draw for DrawSys {
    fn draw(&self, _ctx: &Context) {
        let state = self.store.first::<MenuState>();
        let phase = state.as_ref().map(|s| s.phase).unwrap_or(Phase::Attract);

        self.draw_background();

        match phase {
            Phase::Attract => {
                self.draw_mb_running();
                self.draw_press_enter();
                self.draw_title(get_time() as f32);
            }
            Phase::Transition => {
                self.draw_mb_running();
                self.draw_title(get_time() as f32);
                if let Some(ref s) = state {
                    self.draw_transition(s);
                }
            }
            Phase::Menu | Phase::Selected => {
                // keep the character faintly visible behind the dim
                self.draw_mb_running();
                self.draw_title(get_time() as f32);
                if let Some(ref s) = state {
                    self.draw_menu(s);
                }
            }
        }
    }
}
