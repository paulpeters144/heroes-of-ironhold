use crate::entity::factory_hero::{FRAME_SIZE, SHIELD_SIZE, SWORD_FRAME_SIZE};
use crate::entity::knight::{Facing, Knight, Shield, Sword};
use crate::entity::player::PlayerOne;
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
        let Some(knight_ref) = self
            .store
            .first::<PlayerOne>()
            .and_then(|player| self.store.get_child::<Knight>(&player))
            .map(|k| k.entity_ref())
        else {
            return;
        };

        let (body, frame_offsets, facing) = {
            let Some(knight) = self.store.get_by_id::<Knight>(knight_ref.id()) else {
                return;
            };
            let body = self
                .store
                .get_child::<Animation>(&knight)
                .map(|a| a.position);
            let frame_offsets = self
                .store
                .get_child::<Animation>(&knight)
                .and_then(|a| Knight::offsets().get(a.current_frame).cloned());
            let facing = self
                .store
                .get_child::<Facing>(&knight)
                .map(|f| *f)
                .unwrap_or(Facing::Right);
            (body, frame_offsets, facing)
        };

        let (Some(body), Some(frame_offsets)) = (body, frame_offsets) else {
            return;
        };

        let mirror = facing == Facing::Left;

        let shield_pos = if mirror {
            Vec2::new(
                mirror_x(frame_offsets.shield.x, SHIELD_SIZE),
                frame_offsets.shield.y,
            )
        } else {
            frame_offsets.shield
        };

        let sword_pos = if mirror {
            Vec2::new(
                mirror_x(frame_offsets.sword.x, SWORD_FRAME_SIZE),
                frame_offsets.sword.y,
            )
        } else {
            frame_offsets.sword
        };

        if let Some(anim_ref) = self
            .store
            .get_by_id::<Knight>(knight_ref.id())
            .and_then(|k| self.store.get_child::<Animation>(&k))
            .map(|a| a.entity_ref())
        {
            self.store
                .update::<Animation, _>(&anim_ref, |a| a.flip_x = mirror);
        }

        if let Some(shield_ref) = self
            .store
            .get_by_id::<Knight>(knight_ref.id())
            .and_then(|k| self.store.get_child::<Shield>(&k))
            .map(|s| s.entity_ref())
        {
            if let Some(image_ref) = self
                .store
                .get_by_id::<Shield>(shield_ref.id())
                .and_then(|s| self.store.get_child::<StaticImage>(&s))
                .map(|i| i.entity_ref())
            {
                self.store.update::<StaticImage, _>(&image_ref, |image| {
                    image.position = body + shield_pos;
                    image.visible = frame_offsets.shield_visible;
                    image.flip_x = mirror;
                });
            }
        }

        if let Some(sword_ref) = self
            .store
            .get_by_id::<Knight>(knight_ref.id())
            .and_then(|k| self.store.get_child::<Sword>(&k))
            .map(|s| s.entity_ref())
        {
            if let Some(anim_ref) = self
                .store
                .get_by_id::<Sword>(sword_ref.id())
                .and_then(|s| self.store.get_child::<Animation>(&s))
                .map(|a| a.entity_ref())
            {
                self.store.update::<Animation, _>(&anim_ref, |animation| {
                    animation.position = body + sword_pos;
                    animation.visible = frame_offsets.sword_visible;
                    animation.current_frame = frame_offsets.sword_frame % animation.frame_count;
                    animation.flip_x = mirror;
                });
            }
        }
    }
}
