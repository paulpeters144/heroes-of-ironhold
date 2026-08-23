# macroquad Camera2D Zoom

Macroquad's `Camera2D` zoom is **not** a pixel-space multiplier. It operates in normalized device coordinates (NDC), where the visible range is [-1, 1] in both axes. A zoom of `(1, 1)` means 1 world unit maps to the full NDC range — making the camera show only a 2×2 area regardless of render target resolution.

`Camera2D::from_display_rect(rect)` computes the correct zoom for a 1:1 world-to-pixel mapping as `vec2(2.0 / rect.w, -2.0 / rect.h)`. For a 1280×720 viewport, that's `(0.0015625, -0.002777)`. The Y is negated because macroquad's world Y-down maps to OpenGL's Y-up NDC (with render targets, an additional inversion in the matrix handles the flip).

To apply a user-controlled zoom factor, multiply the base zoom: `camera.zoom = BASE_ZOOM * zoom_factor`. Clamping `zoom_factor` to e.g. `0.5..3.0` gives a reasonable zoom range.
