use crate::systems::Draw;
use crate::{Animation, Context, Drawable, EStore, StaticImage};
use std::rc::Rc;

enum DrawKind {
    Animation(Animation),
    Static(StaticImage),
}

struct DrawCmd {
    z_idx: f32,
    kind: DrawKind,
}

pub struct DrawSystem {
    store: Rc<EStore>,
}

impl DrawSystem {
    pub fn new(store: Rc<EStore>) -> Self {
        Self { store }
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
                z_idx: animation.zdx(),
                kind: DrawKind::Animation(animation.clone()),
            });
        }

        for image in self.store.all::<StaticImage>() {
            if !image.visible {
                continue;
            }
            cmds.push(DrawCmd {
                z_idx: image.zdx(),
                kind: DrawKind::Static(image.clone()),
            });
        }

        cmds.sort_by(|a, b| {
            a.z_idx
                .partial_cmp(&b.z_idx)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for cmd in cmds {
            match cmd.kind {
                DrawKind::Animation(animation) => animation.draw(),
                DrawKind::Static(image) => image.draw(),
            }
        }
    }
}
