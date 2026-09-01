use crate::entity::knight::{
    DASH_DURATION, DASH_SPEED, DOUBLE_TAP_WINDOW, Dash, Knight, IDLE_FRAME, SWIPE_FRAME,
    THRUST_FRAME,
};
use crate::input::{self, Input};
use crate::systems::Update;
use crate::{Animation, Context, EStore};
use macroquad::prelude::Vec2;
use pico_entity_store::store::IntoChild;
use std::rc::Rc;

pub struct PlayerDashSystem {
    store: Rc<EStore>,
    last_tap: Option<Input>,
    tap_timer: f32,
}

impl PlayerDashSystem {
    pub fn new(store: Rc<EStore>) -> Self {
        Self {
            store,
            last_tap: None,
            tap_timer: 0.0,
        }
    }
}

impl Update for PlayerDashSystem {
    fn update(&mut self, ctx: &mut Context) {
        self.tap_timer = (self.tap_timer - ctx.dt).max(0.0);

        let (has_dash, attacking) = {
            let Some(knight) = self.store.first::<Knight>() else {
                return;
            };
            let has_dash = self.store.get_child::<Dash>(&knight).is_some();
            let attacking = self
                .store
                .get_child::<Animation>(&knight)
                .map(|a| matches!(a.current_frame, THRUST_FRAME | SWIPE_FRAME))
                .unwrap_or(false);
            (has_dash, attacking)
        };

        if !attacking && !has_dash {
            let taps = [
                (Input::Left, Vec2::new(-1.0, 0.0)),
                (Input::Right, Vec2::new(1.0, 0.0)),
                (Input::Up, Vec2::new(0.0, -1.0)),
                (Input::Down, Vec2::new(0.0, 1.0)),
            ];
            for (input, dir) in taps {
                if input::down_once(input) {
                    if self.last_tap == Some(input) && self.tap_timer > 0.0 {
                        if let Some(knight) = self.store.first::<Knight>() {
                            let dash = Dash {
                                dir,
                                time: DASH_DURATION,
                            };
                            self.store.add(knight, &[dash.into_child()]);
                        }
                        self.last_tap = None;
                        self.tap_timer = 0.0;
                    } else {
                        self.last_tap = Some(input);
                        self.tap_timer = DOUBLE_TAP_WINDOW;
                    }
                    break;
                }
            }
        }

        let (dir, remaining, dash_ref, anim_ref) = {
            let Some(knight) = self.store.first::<Knight>() else {
                return;
            };
            let Some(dash) = self.store.get_child::<Dash>(&knight) else {
                return;
            };
            let Some(animation) = self.store.get_child::<Animation>(&knight) else {
                return;
            };
            (
                dash.dir,
                dash.time - ctx.dt,
                dash.entity_ref(),
                animation.entity_ref(),
            )
        };

        self.store.update::<Animation, _>(&anim_ref, |animation| {
            animation.position.x += dir.x * DASH_SPEED * ctx.dt;
            animation.position.y += dir.y * DASH_SPEED * ctx.dt;
            animation.current_frame = IDLE_FRAME;
        });

        if remaining <= 0.0 {
            self.store.remove(&[dash_ref]);
        } else {
            self.store.update::<Dash, _>(&dash_ref, |d| d.time = remaining);
        }
    }
}
