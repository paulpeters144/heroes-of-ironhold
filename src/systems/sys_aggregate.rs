use crate::Context;
use std::any::{Any, TypeId};
use std::cell::RefCell;

pub trait System: Any + 'static {
    fn update(&mut self, _ctx: &mut Context) {}

    fn draw(&self, _ctx: &Context) {}

    fn draw_ui(&self, _ctx: &Context) {}
}

pub struct SystemAgg {
    systems: RefCell<Vec<Box<dyn System>>>,
}

impl SystemAgg {
    pub fn new() -> Self {
        SystemAgg {
            systems: RefCell::new(Vec::new()),
        }
    }

    pub fn add<T: System>(&self, system: T) {
        self.systems.borrow_mut().push(Box::new(system));
    }

    pub fn remove<T: System>(&self) -> bool {
        let target = TypeId::of::<T>();
        let index = self
            .systems
            .borrow()
            .iter()
            .position(|s| (**s).type_id() == target);
        if let Some(index) = index {
            self.systems.borrow_mut().remove(index);
            true
        } else {
            false
        }
    }

    pub fn clear(&self) {
        self.systems.borrow_mut().clear();
    }

    pub fn update(&self, ctx: &mut Context) {
        for system in self.systems.borrow_mut().iter_mut() {
            system.update(ctx);
        }
    }

    pub fn draw(&self, ctx: &Context) {
        for system in self.systems.borrow().iter() {
            system.draw(ctx);
        }
    }

    pub fn draw_ui(&self, ctx: &Context) {
        for system in self.systems.borrow().iter() {
            system.draw_ui(ctx);
        }
    }
}

impl Default for SystemAgg {
    fn default() -> Self {
        Self::new()
    }
}
