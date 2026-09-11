use crate::Context;
use std::any::{Any, TypeId};
use std::cell::RefCell;

pub trait Update: Any + 'static {
    fn update(&mut self, ctx: &mut Context);
}

pub trait Draw: Any + 'static {
    fn draw(&self, ctx: &Context);
}

pub trait DrawUi: Any + 'static {
    fn draw_ui(&self, ctx: &Context);
}

pub struct SystemAgg {
    updates: RefCell<Vec<Box<dyn Update>>>,
    draws: RefCell<Vec<Box<dyn Draw>>>,
    uis: RefCell<Vec<Box<dyn DrawUi>>>,
}

impl SystemAgg {
    pub fn new() -> Self {
        SystemAgg {
            updates: RefCell::new(Vec::new()),
            draws: RefCell::new(Vec::new()),
            uis: RefCell::new(Vec::new()),
        }
    }

    pub fn add_update<T: Update>(&self, system: T) {
        self.updates.borrow_mut().push(Box::new(system));
    }

    pub fn add_draw<T: Draw>(&self, system: T) {
        self.draws.borrow_mut().push(Box::new(system));
    }

    pub fn add_ui<T: DrawUi>(&self, system: T) {
        self.uis.borrow_mut().push(Box::new(system));
    }

    pub fn remove_update<T: Update>(&self) -> bool {
        let target = TypeId::of::<T>();
        if let Some(index) = self
            .updates
            .borrow()
            .iter()
            .position(|s| (**s).type_id() == target)
        {
            self.updates.borrow_mut().remove(index);
            true
        } else {
            false
        }
    }

    pub fn remove_draw<T: Draw>(&self) -> bool {
        let target = TypeId::of::<T>();
        if let Some(index) = self
            .draws
            .borrow()
            .iter()
            .position(|s| (**s).type_id() == target)
        {
            self.draws.borrow_mut().remove(index);
            true
        } else {
            false
        }
    }

    pub fn remove_ui<T: DrawUi>(&self) -> bool {
        let target = TypeId::of::<T>();
        if let Some(index) = self
            .uis
            .borrow()
            .iter()
            .position(|s| (**s).type_id() == target)
        {
            self.uis.borrow_mut().remove(index);
            true
        } else {
            false
        }
    }

    pub fn clear(&self) {
        self.updates.borrow_mut().clear();
        self.draws.borrow_mut().clear();
        self.uis.borrow_mut().clear();
    }

    pub fn update(&self, ctx: &mut Context) {
        for system in self.updates.borrow_mut().iter_mut() {
            system.update(ctx);
        }
    }

    pub fn draw(&self, ctx: &Context) {
        for system in self.draws.borrow().iter() {
            system.draw(ctx);
        }
    }

    pub fn draw_ui(&self, ctx: &Context) {
        for system in self.uis.borrow().iter() {
            system.draw_ui(ctx);
        }
    }
}

impl Default for SystemAgg {
    fn default() -> Self {
        Self::new()
    }
}
