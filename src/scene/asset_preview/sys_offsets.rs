use crate::entity::knight::{FrameOffsets, Knight, Shield, Sword};
use crate::systems::Update;
use crate::{Animation, Context, EStore, StaticImage};
use macroquad::prelude::Vec2;

pub struct OffsetUpdateSystem {
    store: &'static EStore,
}

impl OffsetUpdateSystem {
    pub fn new(store: &'static EStore) -> Self {
        Self { store }
    }
}

impl Update for OffsetUpdateSystem {
    fn update(&mut self, _ctx: &mut Context) {
        let mut body: Option<Vec2> = None;
        let mut frame_offsets: Option<FrameOffsets> = None;

        if let Some(knight) = self.store.first::<Knight>() {
            if let Some(animation) = self.store.get_child::<Animation>(&knight) {
                body = Some(animation.position);
                frame_offsets = Knight::offsets()
                    .get(animation.current_frame)
                    .cloned();
            }
        }

        let (Some(body), Some(frame_offsets)) = (body, frame_offsets) else {
            return;
        };

        if let Some(shield) = self.store.first::<Shield>() {
            if let Some(mut image) = self.store.get_child_mut::<StaticImage>(shield) {
                image.position = body + frame_offsets.shield;
                image.visible = frame_offsets.shield_visible;
            }
        }

        if let Some(sword) = self.store.first::<Sword>() {
            if let Some(mut animation) = self.store.get_child_mut::<Animation>(sword) {
                animation.position = body + frame_offsets.sword;
                animation.visible = frame_offsets.sword_visible;
                animation.current_frame = frame_offsets.sword_frame % animation.frame_count;
            }
        }
    }
}
