use crate::entity::factory_hero::{FRAME_SIZE, SHIELD_SIZE, SWORD_FRAME_SIZE};
use crate::entity::knight::{Facing, FrameOffsets, Knight, Shield, Sword};
use crate::systems::Update;
use crate::{Animation, Context, EStore, StaticImage};
use macroquad::prelude::Vec2;
use std::rc::Rc;

pub struct OffsetUpdateSystem {
    store: Rc<EStore>,
}

impl OffsetUpdateSystem {
    pub fn new(store: Rc<EStore>) -> Self {
        Self { store }
    }
}

fn mirror_x(offset_x: f32, child_w: f32) -> f32 {
    FRAME_SIZE - offset_x - child_w
}

impl Update for OffsetUpdateSystem {
    fn update(&mut self, _ctx: &mut Context) {
        let mut body: Option<Vec2> = None;
        let mut frame_offsets: Option<FrameOffsets> = None;
        let mut facing = Facing::Right;

        if let Some(knight) = self.store.first::<Knight>() {
            if let Some(animation) = self.store.get_child::<Animation>(&knight) {
                body = Some(animation.position);
                frame_offsets = Knight::offsets()
                    .get(animation.current_frame)
                    .cloned();
            }
            if let Some(f) = self.store.get_child::<Facing>(&knight) {
                facing = *f;
            }
        }

        let (Some(body), Some(frame_offsets)) = (body, frame_offsets) else {
            return;
        };

        let mirror = facing == Facing::Left;

        let shield_pos = if mirror {
            Vec2::new(mirror_x(frame_offsets.shield.x, SHIELD_SIZE), frame_offsets.shield.y)
        } else {
            frame_offsets.shield
        };

        let sword_pos = if mirror {
            Vec2::new(mirror_x(frame_offsets.sword.x, SWORD_FRAME_SIZE), frame_offsets.sword.y)
        } else {
            frame_offsets.sword
        };

        if let Some(knight) = self.store.first::<Knight>() {
            if let Some(mut animation) = self.store.get_child_mut::<Animation>(knight) {
                animation.flip_x = mirror;
            }
        }

        if let Some(shield) = self.store.first::<Shield>() {
            if let Some(mut image) = self.store.get_child_mut::<StaticImage>(shield) {
                image.position = body + shield_pos;
                image.visible = frame_offsets.shield_visible;
                image.flip_x = mirror;
            }
        }

        if let Some(sword) = self.store.first::<Sword>() {
            if let Some(mut animation) = self.store.get_child_mut::<Animation>(sword) {
                animation.position = body + sword_pos;
                animation.visible = frame_offsets.sword_visible;
                animation.current_frame = frame_offsets.sword_frame % animation.frame_count;
                animation.flip_x = mirror;
            }
        }
    }
}
