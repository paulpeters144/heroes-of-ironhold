use di_container::{BuildContext, Injectable};
use event_bus::EventBus as InnerEventBus;
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;

pub struct EventBus(InnerEventBus);

impl Deref for EventBus {
    type Target = InnerEventBus;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Injectable for EventBus {
    fn inject(
        _ctx: &BuildContext,
    ) -> Pin<Box<dyn Future<Output = di_container::Result<Self>> + '_>> {
        Box::pin(std::future::ready(Ok(EventBus(InnerEventBus::new()))))
    }
}
