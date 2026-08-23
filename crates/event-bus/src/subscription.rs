use std::marker::PhantomData;
use std::rc::Rc;

use crate::{Event, StoredSub};

pub struct Subscription<T: Event> {
    pub(crate) inner: Rc<StoredSub>,
    pub(crate) _phantom: PhantomData<T>,
}

impl<T: Event> Clone for Subscription<T> {
    fn clone(&self) -> Self {
        Subscription {
            inner: self.inner.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<T: Event> Subscription<T> {
    pub fn id(&self) -> usize {
        self.inner.id
    }

    pub fn is_active(&self) -> bool {
        self.inner.active.get()
    }

    pub fn on_message<F: Fn(&T) + 'static>(&self, handler: F) -> &Self {
        let mut guard = self.inner.handler.borrow_mut();
        if guard.is_some() {
            panic!("handler already set on this subscription");
        }
        *guard = Some(Box::new(move |any: &dyn std::any::Any| {
            let event = any
                .downcast_ref::<T>()
                .expect("event-bus: internal type mismatch");
            handler(event);
        }));
        self
    }

    pub fn dispose(&self) {
        if !self.inner.active.replace(false) {
            return;
        }
        self.inner.remove_self();
    }
}

impl<T: Event> Drop for Subscription<T> {
    fn drop(&mut self) {
        self.dispose();
    }
}
