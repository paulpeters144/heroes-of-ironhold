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

/// Rasterization params for crisp UI text.
///
/// UI text is laid out in virtual (640x360) space and scaled to the window by
/// `scale`. macroquad rasterizes each glyph into the font atlas at `font_size`
/// pixels and draws it at `font_size * font_scale`. By rasterizing at the
/// on-screen size (`nominal * scale`) and drawing back down by `1 / scale`, the
/// draw scale cancels the camera zoom, so every atlas texel maps 1:1 to a
/// screen pixel. Glyphs are no longer rasterized small and then upscaled.
pub fn crisp_text_params(nominal_size: u16, scale: f32) -> (u16, f32) {
    let scale = if scale > 0.0 { scale } else { 1.0 };
    let font_size = (nominal_size as f32 * scale).round().max(1.0) as u16;
    (font_size, 1.0 / scale)
}

/// Snap a virtual coordinate to the nearest whole screen pixel. The camera
/// scales virtual space by `scale`, so rounding `v * scale` to an integer and
/// dividing back aligns the point with the screen pixel grid, keeping
/// nearest-filtered text crisp instead of sub-pixel shifted.
pub fn snap_to_pixel(v: f32, scale: f32) -> f32 {
    if scale <= 0.0 {
        return v;
    }
    (v * scale).round() / scale
}
