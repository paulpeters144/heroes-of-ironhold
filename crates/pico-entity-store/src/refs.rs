use parking_lot::{RwLockReadGuard, RwLockWriteGuard};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use crate::entity_ref::EntityRef;
use crate::store::StoreInner;

/// A read guard for a component of type `T`.
///
/// Derefs to `&T`. The guard type `G` defaults to an owned
/// [`RwLockReadGuard`], which keeps the store read-locked for the lifetime of
/// the reference; bulk iteration instead yields references that borrow the
/// iterator's single guard (`G = &StoreInner`).
pub struct Ref<'a, T: 'static, G = RwLockReadGuard<'a, StoreInner>> {
    pub(crate) id: usize,
    pub(crate) data: *const T,
    pub(crate) _guard: G,
    pub(crate) _phantom: PhantomData<(&'a (), T)>,
}

unsafe impl<'a, T: 'static, G> Send for Ref<'a, T, G>
where
    T: Send,
    G: Send,
{
}

unsafe impl<'a, T: 'static, G> Sync for Ref<'a, T, G>
where
    T: Sync,
    G: Sync,
{
}

impl<T: 'static, G: Deref<Target = StoreInner>> Deref for Ref<'_, T, G> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        unsafe { &*self.data }
    }
}

impl<T: 'static, G> Ref<'_, T, G> {
    /// Returns the numeric entity id.
    pub fn id(&self) -> u64 {
        self.id as u64
    }

    /// Converts this reference into a type-erased [`EntityRef`].
    pub fn entity_ref(&self) -> EntityRef {
        EntityRef {
            id: self.id,
            type_id: std::any::TypeId::of::<T>(),
        }
    }
}

/// A write guard for a component of type `T`.
///
/// Derefs to `&mut T`. The guard type `G` defaults to an owned
/// [`RwLockWriteGuard`], which keeps the store write-locked for the lifetime
/// of the reference; bulk iteration instead yields references that borrow the
/// iterator's single guard (`G = &mut StoreInner`).
pub struct RefMut<'a, T: 'static, G = RwLockWriteGuard<'a, StoreInner>> {
    pub(crate) id: usize,
    pub(crate) data: *mut T,
    pub(crate) _guard: G,
    pub(crate) _phantom: PhantomData<(&'a (), T)>,
}

unsafe impl<'a, T: 'static, G> Send for RefMut<'a, T, G>
where
    T: Send,
    G: Send,
{
}

unsafe impl<'a, T: 'static, G> Sync for RefMut<'a, T, G>
where
    T: Sync,
    G: Sync,
{
}

impl<T: 'static, G: Deref<Target = StoreInner>> Deref for RefMut<'_, T, G> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        unsafe { &*self.data }
    }
}

impl<T: 'static, G: DerefMut<Target = StoreInner>> DerefMut for RefMut<'_, T, G> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data }
    }
}

impl<T: 'static, G> RefMut<'_, T, G> {
    /// Returns the numeric entity id.
    pub fn id(&self) -> u64 {
        self.id as u64
    }

    /// Converts this reference into a type-erased [`EntityRef`].
    pub fn entity_ref(&self) -> EntityRef {
        EntityRef {
            id: self.id,
            type_id: std::any::TypeId::of::<T>(),
        }
    }
}

// ── Bulk read iterator (all) ─────────────────────────────────────────────

/// A contiguous read-only view of every component of type `T` in the store.
///
/// Created by [`EntityStore::all`](crate::store::EntityStore::all). Holds a
/// shared read lock for its lifetime; yields a [`Ref`] per component that
/// carries the entity's identity and borrows the iterator's single guard.
pub struct RefVec<'a, T: 'static> {
    guard: RwLockReadGuard<'a, StoreInner>,
    ids: *const u64,
    data: *const T,
    remaining: usize,
    index: usize,
    _phantom: PhantomData<T>,
}

impl<'a, T: 'static> RefVec<'a, T> {
    pub(crate) fn from_raw(
        guard: RwLockReadGuard<'a, StoreInner>,
        ids: *const u64,
        data: *const T,
        len: usize,
    ) -> Self {
        Self {
            guard,
            ids,
            data,
            remaining: len,
            index: 0,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: 'static> Iterator for RefVec<'a, T> {
    type Item = Ref<'a, T, &'a StoreInner>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let slot = self.index;
        let id = unsafe { *self.ids.add(slot) } as usize;
        let store = unsafe { &*(&*self.guard as *const StoreInner) };
        self.remaining -= 1;
        self.index += 1;
        Some(Ref {
            id,
            data: unsafe { self.data.add(slot) },
            _guard: store,
            _phantom: PhantomData,
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }

    fn count(self) -> usize {
        self.remaining
    }
}

impl<'a, T: 'static> std::iter::ExactSizeIterator for RefVec<'a, T> {}

// ── Bulk write iterator (all_mut) ────────────────────────────────────────

/// A contiguous mutable view of every component of type `T` in the store.
///
/// Created by [`EntityStore::all_mut`](crate::store::EntityStore::all_mut).
/// Holds an exclusive write lock for its lifetime; yields a [`RefMut`] per
/// component that carries the entity's identity and borrows the iterator's
/// single guard.
pub struct RefMutVec<'a, T: 'static> {
    guard: RwLockWriteGuard<'a, StoreInner>,
    ids: *const u64,
    data: *mut T,
    remaining: usize,
    index: usize,
    _phantom: PhantomData<T>,
}

impl<'a, T: 'static> RefMutVec<'a, T> {
    pub(crate) fn from_raw(
        guard: RwLockWriteGuard<'a, StoreInner>,
        ids: *const u64,
        data: *mut T,
        len: usize,
    ) -> Self {
        Self {
            guard,
            ids,
            data,
            remaining: len,
            index: 0,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: 'static> Iterator for RefMutVec<'a, T> {
    type Item = RefMut<'a, T, &'a mut StoreInner>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let slot = self.index;
        let id = unsafe { *self.ids.add(slot) } as usize;
        let store = unsafe { &mut *(&mut *self.guard as *mut StoreInner) };
        self.remaining -= 1;
        self.index += 1;
        Some(RefMut {
            id,
            data: unsafe { self.data.add(slot) },
            _guard: store,
            _phantom: PhantomData,
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }

    fn count(self) -> usize {
        self.remaining
    }
}

impl<'a, T: 'static> std::iter::ExactSizeIterator for RefMutVec<'a, T> {}
