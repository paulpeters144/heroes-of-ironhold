use event_bus::EventBus as InnerEventBus;
use std::ops::Deref;

pub struct EventBus(InnerEventBus);

impl EventBus {
    pub fn new() -> Self {
        EventBus(InnerEventBus::new())
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for EventBus {
    type Target = InnerEventBus;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
