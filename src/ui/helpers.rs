use macroquad::prelude::*;

const CORNER_SIDES: u8 = 16;

fn clamp_radius(radius: f32, w: f32, h: f32) -> f32 {
    radius.min(w * 0.5).min(h * 0.5).max(0.0)
}

pub fn draw_rounded_rect(x: f32, y: f32, w: f32, h: f32, radius: f32, color: Color) {
    let r = clamp_radius(radius, w, h);
    if r <= 0.0 {
        draw_rectangle(x, y, w, h, color);
        return;
    }

    draw_rectangle(x + r, y, w - 2.0 * r, h, color);
    draw_rectangle(x, y + r, w, h - 2.0 * r, color);
    draw_circle(x + r, y + r, r, color);
    draw_circle(x + w - r, y + r, r, color);
    draw_circle(x + r, y + h - r, r, color);
    draw_circle(x + w - r, y + h - r, r, color);
}

pub fn draw_rounded_rect_lines(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    radius: f32,
    thickness: f32,
    color: Color,
) {
    let r = clamp_radius(radius, w, h);
    if r <= 0.0 {
        draw_rectangle_lines(x, y, w, h, thickness, color);
        return;
    }

    draw_line(x + r, y, x + w - r, y, thickness, color);
    draw_line(x + r, y + h, x + w - r, y + h, thickness, color);
    draw_line(x, y + r, x, y + h - r, thickness, color);
    draw_line(x + w, y + r, x + w, y + h - r, thickness, color);

    draw_arc(x + r, y + r, CORNER_SIDES, r, 180.0, thickness, 90.0, color);
    draw_arc(
        x + w - r,
        y + r,
        CORNER_SIDES,
        r,
        270.0,
        thickness,
        90.0,
        color,
    );
    draw_arc(
        x + w - r,
        y + h - r,
        CORNER_SIDES,
        r,
        0.0,
        thickness,
        90.0,
        color,
    );
    draw_arc(
        x + r,
        y + h - r,
        CORNER_SIDES,
        r,
        90.0,
        thickness,
        90.0,
        color,
    );
}
