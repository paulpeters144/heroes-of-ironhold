use crate::entity::knight::{Facing, Knight};
use crate::entity::player::PlayerOne;
use crate::systems::{Draw, Update};
use crate::{Animation, Context, EStore};
use macroquad::prelude::*;
use std::rc::Rc;

const ORB_X_OFFSET: f32 = 25.0;

pub struct CameraOrbSystem {
    store: Rc<EStore>,
}

#[derive(Clone, Debug)]
pub struct Orb {
    pub pos: Vec2,
    pub size: f32,
}

impl Orb {
    pub fn new() -> Self {
        Self {
            pos: Vec2::ZERO,
            size: 6.0,
        }
    }
}

impl CameraOrbSystem {
    pub fn new(store: Rc<EStore>) -> Self {
        store.add(Orb::new(), &[]);
        Self { store }
    }

    fn knight_center(&self) -> Option<Vec2> {
        let player = self.store.first::<PlayerOne>()?;
        let knight = self.store.get_child::<Knight>(&player)?;
        let animation = self.store.get_child::<Animation>(&knight)?;
        Some(animation.rect().center())
    }
}

impl Update for CameraOrbSystem {
    fn update(&mut self, _ctx: &mut Context) {
        let Some(center) = self.knight_center() else {
            return;
        };
        let facing_sign = match self
            .store
            .first::<PlayerOne>()
            .and_then(|p| self.store.get_child::<Knight>(&p))
            .and_then(|k| self.store.get_child::<Facing>(&k))
        {
            Some(f) if *f == Facing::Left => -1.0,
            _ => 1.0,
        };
        let pos = center + vec2(ORB_X_OFFSET * facing_sign, 0.0);
        for mut orb in self.store.all_mut::<Orb>() {
            orb.pos = pos;
        }
    }
}

impl Draw for CameraOrbSystem {
    fn draw(&self, _ctx: &Context) {
        let Some(orb) = self.store.first::<Orb>() else {
            return;
        };
        let half = orb.size * 0.5;
        draw_rectangle(
            orb.pos.x - half,
            orb.pos.y - half,
            orb.size,
            orb.size,
            YELLOW,
        );
    }
}
