use crate::input::{self, Input};
use crate::scene::{ChangeSceneEvent, SceneId};
use crate::systems::Update;
use crate::{Context, EStore, EventBus};

pub const FLASH_DUR: f32 = 0.22;
pub const EXPAND_DUR: f32 = 0.35;
pub const HOLD_DUR: f32 = 0.18;
pub const ITEM_STAGGER: f32 = 0.07;
pub const ITEM_SLIDE_DUR: f32 = 0.28;
pub const SELECT_DUR: f32 = 0.45;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Attract,
    Transition,
    Menu,
    Selected,
}

#[derive(Clone, Copy)]
pub struct MenuState {
    pub phase: Phase,
    pub elapsed: f32,
    pub focus: usize,
    pub item_count: usize,
}

impl MenuState {
    pub fn new(item_count: usize) -> Self {
        Self {
            phase: Phase::Attract,
            elapsed: 0.0,
            focus: 0,
            item_count,
        }
    }

    pub fn ease_out_cubic(t: f32) -> f32 {
        1.0 - (1.0 - t).powi(3)
    }

    pub fn ease_out_back(t: f32) -> f32 {
        const C1: f32 = 1.70158;
        const C3: f32 = C1 + 1.0;
        1.0 + C3 * (t - 1.0).powi(3) + C1 * (t - 1.0).powi(2)
    }

    pub fn item_t(&self, index: usize) -> f32 {
        match self.phase {
            Phase::Attract | Phase::Transition => 0.0,
            Phase::Selected => 1.0,
            Phase::Menu => {
                ((self.elapsed - index as f32 * ITEM_STAGGER) / ITEM_SLIDE_DUR).clamp(0.0, 1.0)
            }
        }
    }
}

pub struct MenuSys {
    store: &'static EStore,
    bus: &'static EventBus,
}

impl MenuSys {
    pub fn new(store: &'static EStore, bus: &'static EventBus) -> Self {
        Self { store, bus }
    }
}

impl Update for MenuSys {
    fn update(&mut self, ctx: &mut Context) {
        let dt = ctx.dt;
        let Some(mut state) = self.store.first_mut::<MenuState>() else {
            return;
        };

        state.elapsed += dt;

        match state.phase {
            Phase::Attract => {
                if input::down_once(Input::Enter) {
                    state.phase = Phase::Transition;
                    state.elapsed = 0.0;
                }
            }
            Phase::Transition => {
                let total = FLASH_DUR + EXPAND_DUR + HOLD_DUR;
                if state.elapsed >= total {
                    state.phase = Phase::Menu;
                    state.elapsed = 0.0;
                }
            }
            Phase::Menu => {
                if state.item_count > 1 {
                    if input::down_once(Input::Up) && state.focus > 0 {
                        state.focus -= 1;
                    }
                    if input::down_once(Input::Down) && state.focus + 1 < state.item_count {
                        state.focus += 1;
                    }
                }
                if input::down_once(Input::Enter) {
                    state.phase = Phase::Selected;
                    state.elapsed = 0.0;
                }
            }
            Phase::Selected => {
                if state.elapsed >= SELECT_DUR {
                    if state.focus == 0 {
                        self.bus.fire(&ChangeSceneEvent(SceneId::SceneOne));
                    } else {
                        state.phase = Phase::Menu;
                        state.elapsed = 1.0;
                    }
                }
            }
        }
    }
}
