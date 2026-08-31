use macroquad::prelude::*;

/// Scale factor and centering offset for presenting the logical view so it
/// fills the window as much as possible while preserving aspect ratio.
pub fn view_scale(v_width: f32, v_height: f32) -> (f32, Vec2) {
    let scale = f32::min(screen_width() / v_width, screen_height() / v_height);
    let offset = vec2(
        (screen_width() - v_width * scale) * 0.5,
        (screen_height() - v_height * scale) * 0.5,
    );
    (scale, offset)
}
