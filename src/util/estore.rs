use di_container::{BuildContext, Injectable};
use pico_entity_store::refs::{Ref, RefMut};
use pico_entity_store::store::{ChildSource, EntityStore as PicoEntityStore, IntoAdd};
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;

pub trait Component: 'static {}
impl<T: 'static> Component for T {}

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

    pub fn get_child<Child: 'static>(
        &self,
        parent: &Ref<'_, impl Component>,
    ) -> Option<Ref<'_, Child>> {
        self.children(parent)
            .into_iter()
            .find_map(|child| self.get_by_id::<Child>(child.id()))
    }

    pub fn get_child_mut<Child: 'static>(
        &self,
        parent: Ref<'_, impl Component>,
    ) -> Option<RefMut<'_, Child>> {
        let child_id = self
            .children(&parent)
            .into_iter()
            .find_map(|child| self.get_by_id::<Child>(child.id()).map(|_| child.id()))?;
        drop(parent);
        self.get_by_id_mut::<Child>(child_id)
    }
}

impl Default for EStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Injectable for EStore {
    fn inject(
        _ctx: &BuildContext,
    ) -> Pin<Box<dyn Future<Output = di_container::Result<Self>> + '_>> {
        Box::pin(std::future::ready(Ok(EStore(PicoEntityStore::new()))))
    }
}
