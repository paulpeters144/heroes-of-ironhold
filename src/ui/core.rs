use super::{cmd::Cmd, helpers, params::Label, rect::RectBuilder, style::Style, text::wrap_text};
use crate::{Config, Context};
use macroquad::prelude::*;

pub struct UI {
    pub(super) style: Style,
    pub(super) v_width: f32,
    pub(super) v_height: f32,
    pub mouse_pos: Vec2,
    pub(super) cmds: Vec<Cmd>,
}

impl UI {
    pub fn new(cfg: &Config, style: Style) -> Self {
        UI {
            style,
            v_width: cfg.v_width,
            v_height: cfg.v_height,
            mouse_pos: Vec2::ZERO,
            cmds: Vec::new(),
        }
    }

    pub fn begin(&mut self, _ctx: &Context) {
        self.cmds.clear();
        let scale = f32::min(
            screen_width() / self.v_width,
            screen_height() / self.v_height,
        );
        let offset_x = (screen_width() - self.v_width * scale) * 0.5;
        let offset_y = (screen_height() - self.v_height * scale) * 0.5;
        let (mx, my) = mouse_position();
        self.mouse_pos = vec2((mx - offset_x) / scale, (my - offset_y) / scale);
    }

    pub fn rect(&mut self) -> RectBuilder<'_> {
        RectBuilder { ui: self }
    }

    pub fn label(&mut self, params: Label<'_>) -> Rect {
        let Label {
            font,
            text,
            x,
            y,
            max_width,
            ..
        } = params;
        let text = wrap_text(&font, text, max_width);
        let mut width = 0.0_f32;
        let mut lines = 0u32;
        for line in text.split('\n') {
            let dims = measure_text(line, Some(&font.font), font.size, 1.0);
            width = width.max(dims.width);
            lines += 1;
        }
        let height = lines as f32 * font.size as f32;
        self.cmds.push(Cmd::Text { x, y, text, font });
        Rect::new(x, y, width, height)
    }

    pub fn end(&self) {
        for cmd in &self.cmds {
            match cmd {
                Cmd::Solid {
                    x,
                    y,
                    w,
                    h,
                    color,
                    radius,
                } => helpers::draw_rounded_rect(*x, *y, *w, *h, *radius, *color),
                Cmd::Outlined {
                    x,
                    y,
                    w,
                    h,
                    fill,
                    border,
                    thickness,
                    radius,
                } => {
                    helpers::draw_rounded_rect(*x, *y, *w, *h, *radius, *fill);
                    helpers::draw_rounded_rect_lines(*x, *y, *w, *h, *radius, *thickness, *border);
                }
                Cmd::Text {
                    x,
                    y,
                    ref text,
                    font,
                } => {
                    let mut line_y = *y;
                    for line in text.split('\n') {
                        let dims = measure_text(line, Some(&font.font), font.size, 1.0);
                        draw_text_ex(
                            line,
                            *x,
                            line_y + dims.offset_y,
                            TextParams {
                                font: Some(&font.font),
                                font_size: font.size,
                                font_scale: 1.0,
                                color: font.color,
                                ..Default::default()
                            },
                        );
                        line_y += font.size as f32;
                    }
                }
            }
        }
    }
}
