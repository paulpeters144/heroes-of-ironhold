use std::any::{Any, TypeId};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::{Rc, Weak};

pub trait Event: 'static {}
impl<T: 'static> Event for T {}

type SubMap = Rc<RefCell<HashMap<TypeId, Vec<Rc<StoredSub>>>>>;

mod subscription;
pub use subscription::Subscription;

pub trait AnySub: 'static {}
impl<T: Event> AnySub for Subscription<T> {}

pub struct SubCollection {
    subs: RefCell<Vec<Box<dyn AnySub>>>,
}

impl SubCollection {
    pub fn new() -> Self {
        SubCollection {
            subs: RefCell::new(Vec::new()),
        }
    }

    pub fn push<T: Event>(&self, sub: Subscription<T>) {
        self.subs.borrow_mut().push(Box::new(sub));
    }

    pub fn on<T: Event>(&self, bus: &EventBus, handler: impl Fn(&T) + 'static) {
        let sub = bus.create_sub::<T>();
        sub.on_message(handler);
        self.push(sub);
    }

    pub fn clear(&self) {
        self.subs.borrow_mut().clear();
    }

    pub fn len(&self) -> usize {
        self.subs.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.subs.borrow().is_empty()
    }
}

impl Default for SubCollection {
    fn default() -> Self {
        Self::new()
    }
}

type ErasedHandler = Box<dyn Fn(&dyn Any)>;

pub(crate) struct StoredSub {
    pub id: usize,
    pub event_type: TypeId,
    pub active: Cell<bool>,
    pub handler: RefCell<Option<ErasedHandler>>,
    pub sub_map: Weak<RefCell<HashMap<TypeId, Vec<Rc<StoredSub>>>>>,
}

impl StoredSub {
    pub fn remove_self(&self) {
        if let Some(map) = self.sub_map.upgrade() {
            let mut guard = map.borrow_mut();
            if let Some(subs) = guard.get_mut(&self.event_type) {
                subs.retain(|s| s.id != self.id);
            }
        }
    }
}

pub struct EventBus {
    subscriptions: SubMap,
    next_id: Cell<usize>,
}

impl EventBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn sub_count(&self) -> usize {
        self.subscriptions.borrow().values().map(|v| v.len()).sum()
    }

    pub fn create_sub<T: Event>(&self) -> Subscription<T> {
        let id = self.next_id.get();
        self.next_id.set(id + 1);
        let event_type = TypeId::of::<T>();
        let stored = Rc::new(StoredSub {
            id,
            event_type,
            active: Cell::new(true),
            handler: RefCell::new(None),
            sub_map: Rc::downgrade(&self.subscriptions),
        });

        self.subscriptions
            .borrow_mut()
            .entry(event_type)
            .or_default()
            .push(stored.clone());

        Subscription {
            inner: stored,
            _phantom: PhantomData,
        }
    }

    pub fn fire<T: Event>(&self, event: &T) {
        let any_event = event as &dyn Any;
        let subs = self
            .subscriptions
            .borrow()
            .get(&TypeId::of::<T>())
            .cloned()
            .unwrap_or_default();

        for sub in subs {
            if !sub.active.get() {
                continue;
            }
            let handler_guard = sub.handler.borrow();
            if let Some(ref handler) = *handler_guard {
                handler(any_event);
            }
        }
    }

    pub fn clear(&self) {
        let mut guard = self.subscriptions.borrow_mut();
        *guard = HashMap::new();
    }
}

impl Default for EventBus {
    fn default() -> Self {
        EventBus {
            subscriptions: Rc::new(RefCell::new(HashMap::new())),
            next_id: Cell::new(0),
        }
    }
}
