use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::future::Future;
use std::pin::Pin;

pub use di_container_derive::Injectable;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lifetime {
    Singleton,
    Transient,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    ServiceNotRegistered { type_name: String },
    ServiceConsumed { type_name: String },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ServiceNotRegistered { type_name } => {
                write!(f, "service not registered: {type_name}")
            }
            Error::ServiceConsumed { type_name } => {
                write!(f, "transient service already consumed: {type_name}")
            }
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

pub trait Injectable: Sized {
    fn inject(ctx: &BuildContext) -> Pin<Box<dyn Future<Output = Result<Self>> + '_>>;
}

// ── BuildContext ─────────────────────────────────────────────────────────

pub struct BuildContext {
    singleton_ptrs: RefCell<Vec<*const u8>>,
    transient_values: RefCell<Vec<Option<Box<dyn Any>>>>,
    lifetimes: Vec<Lifetime>,
    type_to_slot: HashMap<TypeId, usize>,
}

impl BuildContext {
    fn new(slot_count: usize) -> Self {
        let mut transient_values = Vec::with_capacity(slot_count);
        for _ in 0..slot_count {
            transient_values.push(None);
        }
        BuildContext {
            singleton_ptrs: RefCell::new(vec![std::ptr::null::<u8>(); slot_count]),
            transient_values: RefCell::new(transient_values),
            lifetimes: vec![Lifetime::Singleton; slot_count],
            type_to_slot: HashMap::new(),
        }
    }

    pub fn get<T: 'static>(&self) -> Result<T> {
        let type_id = TypeId::of::<T>();
        let type_name = std::any::type_name::<T>();

        let &slot_idx =
            self.type_to_slot
                .get(&type_id)
                .ok_or_else(|| Error::ServiceNotRegistered {
                    type_name: type_name.to_string(),
                })?;

        if self.lifetimes[slot_idx] == Lifetime::Singleton {
            let ptr = self.singleton_ptrs.borrow()[slot_idx];
            Ok(unsafe { std::mem::transmute_copy::<*const u8, T>(&ptr) })
        } else {
            let mut values = self.transient_values.borrow_mut();
            let boxed = values[slot_idx]
                .take()
                .ok_or_else(|| Error::ServiceConsumed {
                    type_name: type_name.to_string(),
                })?;
            let typed: Box<T> = boxed
                .downcast::<T>()
                .map_err(|_| Error::ServiceNotRegistered {
                    type_name: type_name.to_string(),
                })?;
            Ok(*typed)
        }
    }
}

// ── Container ────────────────────────────────────────────────────────────

static DUMMY: i32 = 0;

pub struct Container {
    type_to_index: HashMap<TypeId, usize>,
    values: Vec<&'static dyn Any>,
    consumed: Vec<bool>,
}

impl Container {
    pub fn get<T: 'static>(&self) -> Result<&'static T> {
        let type_id = TypeId::of::<T>();
        let type_name = std::any::type_name::<T>();

        let &idx = self
            .type_to_index
            .get(&type_id)
            .ok_or_else(|| Error::ServiceNotRegistered {
                type_name: type_name.to_string(),
            })?;

        if self.consumed[idx] {
            return Err(Error::ServiceConsumed {
                type_name: type_name.to_string(),
            });
        }

        self.values[idx]
            .downcast_ref::<T>()
            .ok_or_else(|| Error::ServiceNotRegistered {
                type_name: type_name.to_string(),
            })
    }
}

// ── ContainerBuilder ─────────────────────────────────────────────────────

type AsyncErasedFactory =
    Box<dyn FnOnce(&BuildContext) -> Pin<Box<dyn Future<Output = Result<Box<dyn Any>>> + '_>>>;

struct DeferredSlot {
    type_ids: Vec<TypeId>,
    lifetime: Lifetime,
    factory: AsyncErasedFactory,
}

pub struct ContainerBuilder {
    slots: Vec<DeferredSlot>,
    type_to_slot: HashMap<TypeId, usize>,
}

impl ContainerBuilder {
    pub fn new() -> Self {
        ContainerBuilder {
            slots: Vec::new(),
            type_to_slot: HashMap::new(),
        }
    }

    pub fn singleton<TService, TImpl>(mut self) -> Self
    where
        TService: 'static,
        TImpl: Injectable + 'static,
    {
        self.add_slot::<TService, TImpl>(Lifetime::Singleton);
        self
    }

    pub fn transient<TService, TImpl>(mut self) -> Self
    where
        TService: 'static,
        TImpl: Injectable + 'static,
    {
        self.add_slot::<TService, TImpl>(Lifetime::Transient);
        self
    }

    fn add_slot<TService, TImpl>(&mut self, lifetime: Lifetime)
    where
        TService: 'static,
        TImpl: Injectable + 'static,
    {
        let slot_idx = self.slots.len();

        let factory: AsyncErasedFactory = Box::new(|ctx: &BuildContext| {
            let fut = TImpl::inject(ctx);
            Box::pin(async move {
                let instance = fut.await?;
                Ok(Box::new(instance) as Box<dyn Any>)
            })
        });

        let type_ids = if lifetime == Lifetime::Singleton {
            vec![TypeId::of::<TImpl>(), TypeId::of::<&TImpl>()]
        } else {
            vec![TypeId::of::<TService>()]
        };

        for &tid in &type_ids {
            self.type_to_slot.insert(tid, slot_idx);
        }

        self.slots.push(DeferredSlot {
            type_ids,
            lifetime,
            factory,
        });
    }

    pub async fn build_and_leak(mut self) -> Result<&'static Container> {
        let slot_count = self.slots.len();

        let mut build_ctx = BuildContext::new(slot_count);
        build_ctx.type_to_slot = self.type_to_slot;

        let slots = std::mem::take(&mut self.slots);
        let mut type_to_index: HashMap<TypeId, usize> = HashMap::new();
        let mut values: Vec<&'static dyn Any> = Vec::with_capacity(slot_count);
        let mut consumed = Vec::with_capacity(slot_count);

        for (idx, slot) in slots.into_iter().enumerate() {
            build_ctx.lifetimes[idx] = slot.lifetime;

            let boxed: Box<dyn Any> = (slot.factory)(&build_ctx).await?;

            if slot.lifetime == Lifetime::Singleton {
                let leaked: &'static dyn Any = Box::leak(boxed);
                let data_ptr = leaked as *const dyn Any as *const u8;
                build_ctx.singleton_ptrs.borrow_mut()[idx] = data_ptr;

                for &tid in &slot.type_ids {
                    type_to_index.insert(tid, values.len());
                }
                values.push(leaked);
                consumed.push(false);
            } else {
                build_ctx.transient_values.borrow_mut()[idx] = Some(boxed);

                for &tid in &slot.type_ids {
                    type_to_index.insert(tid, values.len());
                }
                values.push(&DUMMY as &dyn Any);
                consumed.push(true);
            }
        }

        let container = Box::new(Container {
            type_to_index,
            values,
            consumed,
        });
        Ok(Box::leak(container))
    }
}

impl Default for ContainerBuilder {
    fn default() -> Self {
        Self::new()
    }
}
