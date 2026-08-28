use crate::systems::Draw;
use crate::{Animation, Context, EStore, StaticImage};
use macroquad::prelude::*;

enum DrawKind {
    Animation(Animation),
    Static(StaticImage),
}

struct DrawCmd {
    z_idx: i32,
    kind: DrawKind,
}

pub struct DrawSystem {
    store: &'static EStore,
}

impl DrawSystem {
    pub fn new(store: &'static EStore) -> Self {
        Self { store }
    }

    fn draw_animation(animation: &Animation) {
        let source = Rect::new(
            animation.current_frame as f32 * animation.frame_width,
            0.0,
            animation.frame_width,
            animation.frame_height,
        );
        let dest_size = if animation.dest_size == Vec2::ZERO {
            vec2(animation.frame_width, animation.frame_height)
        } else {
            animation.dest_size
        };

        draw_texture_ex(
            &animation.source,
            animation.position.x,
            animation.position.y,
            animation.tint,
            DrawTextureParams {
                dest_size: Some(dest_size),
                source: Some(source),
                flip_x: animation.flip_x,
                ..Default::default()
            },
        );
    }

    fn draw_static(image: &StaticImage) {
        draw_texture_ex(
            &image.source,
            image.position.x,
            image.position.y,
            image.tint,
            DrawTextureParams {
                dest_size: Some(image.size),
                flip_x: image.flip_x,
                ..Default::default()
            },
        );
    }
}

impl Draw for DrawSystem {
    fn draw(&self, _ctx: &Context) {
        let mut cmds: Vec<DrawCmd> = Vec::new();

        for animation in self.store.all::<Animation>() {
            if !animation.visible {
                continue;
            }
            cmds.push(DrawCmd {
                z_idx: animation.z_idx,
                kind: DrawKind::Animation(animation.clone()),
            });
        }

        for image in self.store.all::<StaticImage>() {
            if !image.visible {
                continue;
            }
            cmds.push(DrawCmd {
                z_idx: image.z_idx,
                kind: DrawKind::Static(image.clone()),
            });
        }

        cmds.sort_by_key(|cmd| cmd.z_idx);

        for cmd in cmds {
            match cmd.kind {
                DrawKind::Animation(animation) => Self::draw_animation(&animation),
                DrawKind::Static(image) => Self::draw_static(&image),
            }
        }
    }
}
