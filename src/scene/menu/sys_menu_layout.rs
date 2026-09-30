use crate::entity::{Animation, StaticImage};
use crate::prelude::*;
use macroquad::prelude::*;
use std::rc::Rc;

// The menu is a hand-composed 640x360 diorama. `frame` maps that composition
// onto the current view: `k` is the factor by which the background covers the
// view (cropping any overflow), and `origin` is where the composition's (0,0)
// lands. Props keep their 1:1 pixel size; only their positions scale by `k`,
// so the diorama stays glued to the background painting at any window size.
pub(super) const COMP_W: f32 = 640.0;
pub(super) const COMP_H: f32 = 360.0;
pub(super) const SCROLL_BASE: (f32, f32) = (2.0, 135.0);

pub(super) fn frame(view: Vec2) -> (Vec2, f32) {
    let k = f32::max(view.x / COMP_W, view.y / COMP_H);
    let origin = vec2((view.x - COMP_W * k) * 0.5, (view.y - COMP_H * k) * 0.5);
    (origin, k)
}

/// Marks the menu background so the layout system can cover-scale it.
#[derive(Clone, Debug)]
pub struct MenuCover;

/// Marks a menu decoration. Its position in the 640x360 composition is
/// captured from the spawned position on the first frame.
#[derive(Clone, Debug, Default)]
pub struct MenuProp {
    base: Option<Vec2>,
}

pub struct MenuLayoutSystem {
    store: Rc<EStore>,
}

impl MenuLayoutSystem {
    pub fn new(store: Rc<EStore>) -> Self {
        Self { store }
    }
}

impl System for MenuLayoutSystem {
    fn update(&mut self, ctx: &mut Context) {
        let (origin, k) = frame(ctx.view.view);

        // Collect first; mutating while iterating would double-borrow a storage.
        let mut covers = Vec::new();
        for cover in self.store.all::<MenuCover>() {
            if let Some(parent) = self.store.parent(&cover) {
                covers.push(parent);
            }
        }
        let mut props = Vec::new();
        for prop in self.store.all::<MenuProp>() {
            if let Some(parent) = self.store.parent(&prop) {
                props.push((prop.entity_ref(), parent, prop.base));
            }
        }

        for parent in covers {
            self.store.update::<StaticImage, _>(&parent, |s| {
                s.position = origin;
                s.size = vec2(COMP_W * k, COMP_H * k);
            });
        }

        for (prop_ref, parent, base) in props {
            let base = match base {
                Some(base) => base,
                None => {
                    let mut found = Vec2::ZERO;
                    if !self
                        .store
                        .update::<StaticImage, _>(&parent, |s| found = s.position)
                    {
                        self.store
                            .update::<Animation, _>(&parent, |a| found = a.position);
                    }
                    self.store
                        .update::<MenuProp, _>(&prop_ref, |p| p.base = Some(found));
                    found
                }
            };
            let position = origin + base * k;
            if !self
                .store
                .update::<StaticImage, _>(&parent, |s| s.position = position)
            {
                self.store
                    .update::<Animation, _>(&parent, |a| a.position = position);
            }
        }
    }
}
