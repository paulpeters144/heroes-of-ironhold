use crate::entity::enemy::RamHead;
use crate::entity::knight::Knight;
use crate::systems::{Draw, Update};
use crate::{Animation, CollisionRect, Context, EStore};
use macroquad::prelude::{draw_rectangle_lines, vec2, Color};
use pico_entity_store::entity_ref::EntityRef;
use std::rc::Rc;

pub struct CollisionRectSystem {
    store: Rc<EStore>,
}

impl CollisionRectSystem {
    pub fn new(store: Rc<EStore>) -> Self {
        Self { store }
    }

    fn sync_marker<T: 'static>(&self) {
        let markers: Vec<EntityRef> = self
            .store
            .all::<T>()
            .map(|m| m.entity_ref())
            .collect();

        for marker in markers {
            let Some(center) = self
                .store
                .get_by_id::<T>(marker.id())
                .and_then(|m| self.store.get_child::<Animation>(&m))
                .map(|a| a.rect())
                .map(|r| vec2(r.x + r.w / 2.0, r.y + r.h / 2.0))
            else {
                continue;
            };

            let Some(rect_ref) = self
                .store
                .get_by_id::<T>(marker.id())
                .and_then(|m| self.store.get_child::<CollisionRect>(&m))
                .map(|r| r.entity_ref())
            else {
                continue;
            };

            self.store.update::<CollisionRect, _>(&rect_ref, |r| {
                r.rect.x = center.x - r.rect.w / 2.0;
                r.rect.y = center.y - r.rect.h / 2.0;
            });
        }
    }
}

impl Update for CollisionRectSystem {
    fn update(&mut self, _ctx: &mut Context) {
        self.sync_marker::<Knight>();
        self.sync_marker::<RamHead>();
    }
}

impl Draw for CollisionRectSystem {
    fn draw(&self, ctx: &Context) {
        if !ctx.debug {
            return;
        }

        for rect in self.store.all::<CollisionRect>() {
            draw_rectangle_lines(
                rect.rect.x,
                rect.rect.y,
                rect.rect.w,
                rect.rect.h,
                2.0,
                Color::new(1.0, 0.0, 0.0, 1.0),
            );
        }
    }
}
