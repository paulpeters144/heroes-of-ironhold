use macroquad::prelude::{Image, Rect};

/// A single-frame sprite's CPU pixel data plus its world-space placement.
///
/// The sprite is drawn as a uniform `scale` of the source image, so its world
/// size is `image.width() * scale` x `image.height() * scale`. When `flip_x` is
/// set the sprite is mirrored horizontally and the x mapping is reversed.
#[derive(Clone)]
pub struct ImageData {
    /// CPU-side single-frame sprite pixel data (alpha channel used for the hit test).
    pub image: Image,
    /// World-space left edge of the sprite image.
    pub x: f32,
    /// World-space top edge of the sprite image.
    pub y: f32,
    /// Uniform scale the sprite is drawn at.
    pub scale: f32,
    /// True if the sprite is drawn horizontally flipped; `did_attack` mirrors the x mapping.
    pub flip_x: bool,
}

/// Returns true if the attack rect overlaps any pixel of the sprite whose alpha is > 0.
pub fn did_attack(attack_rect: Rect, data: &ImageData) -> bool {
    if data.scale <= 0.0 {
        return false;
    }

    let width = data.image.width() as f32;
    let height = data.image.height() as f32;
    let world_w = width * data.scale;
    let world_h = height * data.scale;

    let ox0 = f32::max(attack_rect.x, data.x);
    let oy0 = f32::max(attack_rect.y, data.y);
    let ox1 = f32::min(attack_rect.x + attack_rect.w, data.x + world_w);
    let oy1 = f32::min(attack_rect.y + attack_rect.h, data.y + world_h);
    if ox0 >= ox1 || oy0 >= oy1 {
        return false;
    }

    let scale = data.scale;
    let sx0 = ((ox0 - data.x) / scale).floor().max(0.0) as usize;
    let sx1 = (((ox1 - data.x) / scale).ceil().max(0.0) as usize).min(width as usize);
    let sy0 = ((oy0 - data.y) / scale).floor().max(0.0) as usize;
    let sy1 = (((oy1 - data.y) / scale).ceil().max(0.0) as usize).min(height as usize);

    let pixels = data.image.get_image_data();
    let stride = width as usize;
    let w = width as usize;

    for sy in sy0..sy1 {
        for sx in sx0..sx1 {
            let px = if data.flip_x { w - 1 - sx } else { sx };
            if pixels[sy * stride + px][3] > 0 {
                return true;
            }
        }
    }
    false
}
