use pico_entity_store::store::{ChildSource, EntityStore as PicoEntityStore, IntoAdd};
use std::ops::Deref;

pub struct EStore(PicoEntityStore);

impl Deref for EStore {
    type Target = PicoEntityStore;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl EStore {
    pub fn new() -> Self {
        EStore(PicoEntityStore::new())
    }

    pub fn add<T: 'static + Clone + Send + Sync>(
        &self,
        target: impl IntoAdd<T>,
        children: &[ChildSource],
    ) {
        let _ = self.0.add(target, children);
    }
}

impl Default for EStore {
    fn default() -> Self {
        Self::new()
    }
}
