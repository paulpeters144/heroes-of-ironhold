use crate::Animation;
use macroquad::prelude::{Image, Rect, Texture2D, Vec2};

/// A sprite sheet's CPU pixel data plus its world-space placement.
///
/// The sprite is drawn as a uniform `scale` of one frame, so its world size is
/// `frame_width * scale` x `frame_height * scale`. `current_frame` selects which
/// single-row frame `did_attack` samples; when `flip_x` is set the x mapping is
/// mirrored within that frame.
#[derive(Clone)]
pub struct ImageData {
    /// CPU-side sprite sheet pixel data (alpha channel used for the hit test).
    pub image: Image,
    /// World-space left edge of the current frame.
    pub x: f32,
    /// World-space top edge of the current frame.
    pub y: f32,
    /// World px per frame pixel.
    pub scale: f32,
    /// True if the sprite is drawn horizontally flipped; `did_attack` mirrors x.
    pub flip_x: bool,
    /// Frame width in sheet pixels.
    pub frame_width: f32,
    /// Frame height in sheet pixels.
    pub frame_height: f32,
    /// Which frame within the sheet is currently shown.
    pub current_frame: usize,
}

/// Builds `ImageData` for an animation's current frame, reusing a cached CPU
/// copy of the sheet while the texture handle is unchanged.
pub fn image_data_for(animation: &Animation, cache: &mut Option<(Texture2D, Image)>) -> ImageData {
    let image = match cache {
        Some((texture, image)) if *texture == animation.source => image.clone(),
        _ => {
            let image = animation.source.get_texture_data();
            *cache = Some((animation.source.clone(), image.clone()));
            image
        }
    };

    let dest = if animation.dest_size == Vec2::ZERO {
        Vec2::new(animation.frame_width, animation.frame_height)
    } else {
        animation.dest_size
    };
    let scale = dest.x * animation.scale / animation.frame_width;

    ImageData {
        image,
        x: animation.position.x,
        y: animation.position.y,
        scale,
        flip_x: animation.flip_x,
        frame_width: animation.frame_width,
        frame_height: animation.frame_height,
        current_frame: animation.current_frame,
    }
}

/// Returns true if `attack_rect` overlaps any pixel (alpha > 0) of the frame
/// identified by `frame_width`/`frame_height`/`current_frame` within
/// `ImageData.image`, mapping world coordinates into the frame and mirroring
/// `flip_x` within the frame.
pub fn did_attack(attack_rect: Rect, data: &ImageData) -> bool {
    if data.scale <= 0.0 || data.frame_width <= 0.0 || data.frame_height <= 0.0 {
        return false;
    }

    let world_w = data.frame_width * data.scale;
    let world_h = data.frame_height * data.scale;

    let ox0 = f32::max(attack_rect.x, data.x);
    let oy0 = f32::max(attack_rect.y, data.y);
    let ox1 = f32::min(attack_rect.x + attack_rect.w, data.x + world_w);
    let oy1 = f32::min(attack_rect.y + attack_rect.h, data.y + world_h);
    if ox0 >= ox1 || oy0 >= oy1 {
        return false;
    }

    let scale = data.scale;
    let frame_w = data.frame_width as usize;
    let frame_h = data.frame_height as usize;

    let sx0 = ((ox0 - data.x) / scale).floor().max(0.0) as usize;
    let sx1 = (((ox1 - data.x) / scale).ceil().max(0.0) as usize).min(frame_w);
    let sy0 = ((oy0 - data.y) / scale).floor().max(0.0) as usize;
    let sy1 = (((oy1 - data.y) / scale).ceil().max(0.0) as usize).min(frame_h);

    let sheet_w = data.image.width();
    let sheet_h = data.image.height();
    let frame_x0 = (data.current_frame * frame_w).min(sheet_w.saturating_sub(frame_w));

    let pixels = data.image.get_image_data();

    for sy in sy0..sy1 {
        if sy >= sheet_h {
            break;
        }
        for sx in sx0..sx1 {
            let px = if data.flip_x {
                frame_x0 + (frame_w - 1 - sx)
            } else {
                frame_x0 + sx
            };
            if px >= sheet_w {
                continue;
            }
            if pixels[sy * sheet_w + px][3] > 0 {
                return true;
            }
        }
    }
    false
}
