//! This module describes traits for implementing [`LockerRoom`](crate::LockerRoom)'s and [`LockerRoomAsync`](crate::LockerRoomAsync)'s
//! functionality for a certain collection.

use std::{
    borrow::Borrow,
    collections::{BTreeMap, HashMap, VecDeque},
    hash::Hash,
    sync::RwLock,
};

/// Trait describes functionality of collection that necessary for creating [`LockerRoom`](crate::LockerRoom)
/// and [`LockerRoomAsync`](crate::LockerRoomAsync).
pub trait Collection {
    /// Type that should be used as index
    type Idx;
    /// The returned type after indexing.
    type Output: ?Sized;
    /// Type of collection which stores [`RwLock`]s. Usually the same type as `Collection`'s implementor.
    ///
    /// It's necessary because of performance. For example, implementing Collection for [`Vec`] but using [`BTreeMap`] as ShadowLocks makes little sense
    /// because [`LockerRoom`](crate::LockerRoom) at every [`index`](Self::index) (or [`index_mut`](Self::index_mut)) method call will also call the
    /// same method of ShadowLocks. This makes meaningless to use `Vec` because its performance will be bottlenecked by `BTreeMap`.
    type ShadowLocks: ShadowLocksCollection<Idx = Self::Idx>;
    #[cfg(any(feature = "async", doc))]
    #[doc(cfg(feature = "async"))]
    /// Similar to [`ShadowLocks`](Collection::ShadowLocks) but stores tokio's [`RwLock`](tokio::sync::RwLock)s.
    ///
    /// Used in [`LockerRoomAsync`](crate::LockerRoomAsync).
    type ShadowLocksAsync: ShadowLocksCollectionAsync<Idx = Self::Idx>;

    /// Performs the indexing operation. But unlike the [`Index::index`](std::ops::Index::index), it doesn't panic, and return None.
    fn index(&self, index: impl Borrow<Self::Idx>) -> Option<&Self::Output>;
    /// Performs the mutable indexing operation. But unlike the [`IndexMut::index_mut`](std::ops::IndexMut::index_mut), it doesn't panic, and return None.
    fn index_mut(&mut self, index: impl Borrow<Self::Idx>) -> Option<&mut Self::Output>;
    /// An iterator visiting all indices.
    fn indices(&self) -> impl Iterator<Item = Self::Idx>;
    /// Creates collection which stores [`RwLock`]s.
    ///
    /// Used in [`LockerRoom`](crate::LockerRoom).
    fn shadow_locks(&self) -> Self::ShadowLocks;
    #[cfg(any(feature = "async", doc))]
    #[doc(cfg(feature = "async"))]
    /// Creates collection wich stores tokio's [`RwLock`](tokio::sync::RwLock)s.
    ///
    /// Used in [`LockerRoomAsync`](crate::LockerRoomAsync).
    fn shadow_locks_async(&self) -> Self::ShadowLocksAsync;
}

impl<T> Collection for [T] {
    type Idx = usize;
    type Output = T;
    type ShadowLocks = Vec<RwLock<()>>;
    #[cfg(any(feature = "async", doc))]
    #[doc(cfg(feature = "async"))]
    type ShadowLocksAsync = Vec<tokio::sync::RwLock<()>>;

    fn index(&self, index: impl Borrow<Self::Idx>) -> Option<&Self::Output> {
        self.get(*index.borrow())
    }

    fn index_mut(&mut self, index: impl Borrow<Self::Idx>) -> Option<&mut Self::Output> {
        self.get_mut(*index.borrow())
    }

    fn indices(&self) -> impl Iterator<Item = Self::Idx> {
        0..self.len()
    }

    fn shadow_locks(&self) -> Self::ShadowLocks {
        self.indices().map(|_| RwLock::new(())).collect::<Vec<_>>()
    }

    #[cfg(any(feature = "async", doc))]
    #[doc(cfg(feature = "async"))]
    fn shadow_locks_async(&self) -> Self::ShadowLocksAsync {
        self.indices()
            .map(|_| tokio::sync::RwLock::new(()))
            .collect::<Vec<_>>()
    }
}

impl<T, const N: usize> Collection for [T; N] {
    type Idx = usize;
    type Output = T;
    type ShadowLocks = Vec<RwLock<()>>;
    #[cfg(any(feature = "async", doc))]
    #[doc(cfg(feature = "async"))]
    type ShadowLocksAsync = Vec<tokio::sync::RwLock<()>>;

    fn index(&self, index: impl Borrow<Self::Idx>) -> Option<&Self::Output> {
        self.get(*index.borrow())
    }

    fn index_mut(&mut self, index: impl Borrow<Self::Idx>) -> Option<&mut Self::Output> {
        self.get_mut(*index.borrow())
    }

    fn indices(&self) -> impl Iterator<Item = Self::Idx> {
        0..self.len()
    }

    fn shadow_locks(&self) -> Self::ShadowLocks {
        self.indices().map(|_| RwLock::new(())).collect::<Vec<_>>()
    }

    #[cfg(any(feature = "async", doc))]
    #[doc(cfg(feature = "async"))]
    fn shadow_locks_async(&self) -> Self::ShadowLocksAsync {
        self.indices()
            .map(|_| tokio::sync::RwLock::new(()))
            .collect::<Vec<_>>()
    }
}

impl<T> Collection for Vec<T> {
    type Idx = usize;
    type Output = T;
    type ShadowLocks = Vec<RwLock<()>>;
    #[cfg(any(feature = "async", doc))]
    #[doc(cfg(feature = "async"))]
    type ShadowLocksAsync = Vec<tokio::sync::RwLock<()>>;

    fn index(&self, index: impl Borrow<Self::Idx>) -> Option<&Self::Output> {
        self.get(*index.borrow())
    }

    fn index_mut(&mut self, index: impl Borrow<Self::Idx>) -> Option<&mut Self::Output> {
        self.get_mut(*index.borrow())
    }

    fn indices(&self) -> impl Iterator<Item = Self::Idx> {
        0..self.len()
    }

    fn shadow_locks(&self) -> Self::ShadowLocks {
        self.indices().map(|_| RwLock::new(())).collect::<Vec<_>>()
    }

    #[cfg(any(feature = "async", doc))]
    #[doc(cfg(feature = "async"))]
    fn shadow_locks_async(&self) -> Self::ShadowLocksAsync {
        self.indices()
            .map(|_| tokio::sync::RwLock::new(()))
            .collect::<Vec<_>>()
    }
}

impl<T> Collection for VecDeque<T> {
    type Idx = usize;
    type Output = T;
    type ShadowLocks = VecDeque<RwLock<()>>;
    #[cfg(any(feature = "async", doc))]
    #[doc(cfg(feature = "async"))]
    type ShadowLocksAsync = VecDeque<tokio::sync::RwLock<()>>;

    fn index(&self, index: impl Borrow<Self::Idx>) -> Option<&Self::Output> {
        self.get(*index.borrow())
    }

    fn index_mut(&mut self, index: impl Borrow<Self::Idx>) -> Option<&mut Self::Output> {
        self.get_mut(*index.borrow())
    }

    fn indices(&self) -> impl Iterator<Item = Self::Idx> {
        0..self.len()
    }

    fn shadow_locks(&self) -> Self::ShadowLocks {
        self.indices()
            .map(|_| RwLock::new(()))
            .collect::<VecDeque<_>>()
    }

    #[cfg(any(feature = "async", doc))]
    #[doc(cfg(feature = "async"))]
    fn shadow_locks_async(&self) -> Self::ShadowLocksAsync {
        self.indices()
            .map(|_| tokio::sync::RwLock::new(()))
            .collect::<VecDeque<_>>()
    }
}

impl<K, V> Collection for HashMap<K, V>
where
    K: Eq + Hash + Clone + ?Sized,
{
    type Idx = K;
    type Output = V;
    type ShadowLocks = HashMap<Self::Idx, RwLock<()>>;
    #[cfg(any(feature = "async", doc))]
    #[doc(cfg(feature = "async"))]
    type ShadowLocksAsync = HashMap<Self::Idx, tokio::sync::RwLock<()>>;

    fn index(&self, index: impl Borrow<Self::Idx>) -> Option<&Self::Output> {
        self.get(index.borrow())
    }

    fn index_mut(&mut self, index: impl Borrow<Self::Idx>) -> Option<&mut Self::Output> {
        self.get_mut(index.borrow())
    }

    fn indices(&self) -> impl Iterator<Item = Self::Idx> {
        self.keys().cloned()
    }

    fn shadow_locks(&self) -> Self::ShadowLocks {
        self.indices()
            .map(|index| (index, RwLock::new(())))
            .collect::<HashMap<_, _>>()
    }

    #[cfg(any(feature = "async", doc))]
    #[doc(cfg(feature = "async"))]
    fn shadow_locks_async(&self) -> Self::ShadowLocksAsync {
        self.indices()
            .map(|index| (index, tokio::sync::RwLock::new(())))
            .collect::<HashMap<_, _>>()
    }
}

impl<K, V> Collection for BTreeMap<K, V>
where
    K: Ord + Clone + ?Sized,
{
    type Idx = K;
    type Output = V;
    type ShadowLocks = BTreeMap<Self::Idx, RwLock<()>>;
    #[cfg(any(feature = "async", doc))]
    #[doc(cfg(feature = "async"))]
    type ShadowLocksAsync = BTreeMap<Self::Idx, tokio::sync::RwLock<()>>;

    fn index(&self, index: impl Borrow<Self::Idx>) -> Option<&Self::Output> {
        self.get(index.borrow())
    }

    fn index_mut(&mut self, index: impl Borrow<Self::Idx>) -> Option<&mut Self::Output> {
        self.get_mut(index.borrow())
    }

    fn indices(&self) -> impl Iterator<Item = Self::Idx> {
        self.keys().cloned()
    }

    fn shadow_locks(&self) -> Self::ShadowLocks {
        self.indices()
            .map(|index| (index, RwLock::new(())))
            .collect::<BTreeMap<_, _>>()
    }

    #[cfg(any(feature = "async", doc))]
    #[doc(cfg(feature = "async"))]
    fn shadow_locks_async(&self) -> Self::ShadowLocksAsync {
        self.indices()
            .map(|index| (index, tokio::sync::RwLock::new(())))
            .collect::<BTreeMap<_, _>>()
    }
}

/// Specifies structures that can be used as [`Collection::ShadowLocks`].
pub trait ShadowLocksCollection {
    /// Type that should be used as index.
    type Idx;

    /// Performs the indexing operation returning lock for the cell.
    fn index(&self, index: impl Borrow<Self::Idx>) -> Option<&RwLock<()>>;
    /// Update internal state to store [`RwLock`]'s with new indices.
    fn update_indices(&mut self, indices: impl Iterator<Item = Self::Idx>);
}

impl ShadowLocksCollection for Vec<RwLock<()>> {
    type Idx = usize;

    fn index(&self, index: impl Borrow<Self::Idx>) -> Option<&RwLock<()>> {
        self.get(*index.borrow())
    }

    fn update_indices(&mut self, indices: impl Iterator<Item = Self::Idx>) {
        self.resize_with(indices.count(), || RwLock::new(()));
    }
}

impl ShadowLocksCollection for VecDeque<RwLock<()>> {
    type Idx = usize;

    fn index(&self, index: impl Borrow<Self::Idx>) -> Option<&RwLock<()>> {
        self.get(*index.borrow())
    }

    fn update_indices(&mut self, indices: impl Iterator<Item = Self::Idx>) {
        self.resize_with(indices.count(), || RwLock::new(()));
    }
}

impl<K> ShadowLocksCollection for HashMap<K, RwLock<()>>
where
    K: Eq + Hash + Clone + ?Sized,
{
    type Idx = K;

    fn index(&self, index: impl Borrow<Self::Idx>) -> Option<&RwLock<()>> {
        self.get(index.borrow())
    }

    fn update_indices(&mut self, indices: impl Iterator<Item = Self::Idx>) {
        self.clear();
        self.extend(indices.map(|index| (index, RwLock::new(()))));
    }
}

impl<K> ShadowLocksCollection for BTreeMap<K, RwLock<()>>
where
    K: Ord + Clone + ?Sized,
{
    type Idx = K;

    fn index(&self, index: impl Borrow<Self::Idx>) -> Option<&RwLock<()>> {
        self.get(index.borrow())
    }

    fn update_indices(&mut self, indices: impl Iterator<Item = Self::Idx>) {
        self.clear();
        self.extend(indices.map(|index| (index, RwLock::new(()))));
    }
}

#[cfg(any(feature = "async", doc))]
#[doc(cfg(feature = "async"))]
/// Specifies structures that can be used as [`Collection::ShadowLocksAsync`].
pub trait ShadowLocksCollectionAsync {
    /// Type that should be used as index.
    type Idx;

    /// Performs the indexing operation lock for the cell.
    fn index(&self, index: impl Borrow<Self::Idx>) -> Option<&tokio::sync::RwLock<()>>;
    /// Update internal state to store tokio's [`RwLock`](tokio::sync::RwLock)'s with new indices.
    fn update_indices(&mut self, indices: impl Iterator<Item = Self::Idx>);
}

#[cfg(any(feature = "async", doc))]
#[doc(cfg(feature = "async"))]
impl ShadowLocksCollectionAsync for Vec<tokio::sync::RwLock<()>> {
    type Idx = usize;

    fn index(&self, index: impl Borrow<Self::Idx>) -> Option<&tokio::sync::RwLock<()>> {
        self.get(*index.borrow())
    }

    fn update_indices(&mut self, indices: impl Iterator<Item = Self::Idx>) {
        self.resize_with(indices.count(), || tokio::sync::RwLock::new(()));
    }
}

#[cfg(any(feature = "async", doc))]
#[doc(cfg(feature = "async"))]
impl ShadowLocksCollectionAsync for VecDeque<tokio::sync::RwLock<()>> {
    type Idx = usize;

    fn index(&self, index: impl Borrow<Self::Idx>) -> Option<&tokio::sync::RwLock<()>> {
        self.get(*index.borrow())
    }

    fn update_indices(&mut self, indices: impl Iterator<Item = Self::Idx>) {
        self.resize_with(indices.count(), || tokio::sync::RwLock::new(()));
    }
}

#[cfg(any(feature = "async", doc))]
#[doc(cfg(feature = "async"))]
impl<K> ShadowLocksCollectionAsync for HashMap<K, tokio::sync::RwLock<()>>
where
    K: Eq + Hash + Clone + ?Sized,
{
    type Idx = K;

    fn index(&self, index: impl Borrow<Self::Idx>) -> Option<&tokio::sync::RwLock<()>> {
        self.get(index.borrow())
    }

    fn update_indices(&mut self, indices: impl Iterator<Item = Self::Idx>) {
        self.clear();
        self.extend(indices.map(|index| (index, tokio::sync::RwLock::new(()))));
    }
}

#[cfg(any(feature = "async", doc))]
#[doc(cfg(feature = "async"))]
impl<K> ShadowLocksCollectionAsync for BTreeMap<K, tokio::sync::RwLock<()>>
where
    K: Ord + Clone + ?Sized,
{
    type Idx = K;

    fn index(&self, index: impl Borrow<Self::Idx>) -> Option<&tokio::sync::RwLock<()>> {
        self.get(index.borrow())
    }

    fn update_indices(&mut self, indices: impl Iterator<Item = Self::Idx>) {
        self.clear();
        self.extend(indices.map(|index| (index, tokio::sync::RwLock::new(()))));
    }
}
