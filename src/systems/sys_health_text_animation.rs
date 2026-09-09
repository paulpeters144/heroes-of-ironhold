use crate::events::HealthChangeEvent;
use crate::systems::Update;
use crate::{Assets, Context, EStore, EventBus, FloatingText, FontTag, SubCollection, TextStyle};
use macroquad::prelude::{Color, Font};
use pico_entity_store::entity_ref::EntityRef;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

const DAMAGE_COLOR: Color = Color::new(1.0, 0.2, 0.14, 1.0);
const HEAL_COLOR: Color = Color::new(0.25, 0.9, 0.35, 1.0);
const FONT_SIZE: u16 = 12;

pub struct HealthTextAnimationSystem {
    store: Rc<EStore>,
    queue: Rc<RefCell<VecDeque<HealthChangeEvent>>>,
    font: Font,
    _subs: Rc<SubCollection>,
}

impl HealthTextAnimationSystem {
    /// Subscribes to `HealthChangeEvent` and extracts the Pixellari font (size 24) once at construction.
    pub fn new(store: Rc<EStore>, bus: Rc<EventBus>, assets: &Assets) -> Self {
        let queue = Rc::new(RefCell::new(VecDeque::new()));
        let subs = Rc::new(SubCollection::new());
        let queue_for_handler = queue.clone();
        subs.on::<HealthChangeEvent>(&bus, move |event: &HealthChangeEvent| {
            queue_for_handler.borrow_mut().push_back(event.clone());
        });
        let base = assets.get_font(&TextStyle::new(FontTag::Body));
        Self {
            store,
            queue,
            font: base.font,
            _subs: subs,
        }
    }
}

impl Update for HealthTextAnimationSystem {
    fn update(&mut self, ctx: &mut Context) {
        while let Some(event) = self.queue.borrow_mut().pop_front() {
            let (color, text) = if event.amount >= 0 {
                (HEAL_COLOR, format!("+{}", event.amount))
            } else {
                (DAMAGE_COLOR, event.amount.to_string())
            };
            self.store.add(
                FloatingText::new(text, event.rect, self.font.clone(), FONT_SIZE, color),
                &[],
            );
        }

        let dt = ctx.dt;
        let mut expired: Vec<EntityRef> = Vec::new();
        for mut text in self.store.all_mut::<FloatingText>() {
            text.age += dt;
            if text.age >= text.lifetime {
                expired.push(text.entity_ref());
            }
        }
        if !expired.is_empty() {
            self.store.remove(&expired);
        }
    }
}
