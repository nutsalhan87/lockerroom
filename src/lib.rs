//! Crate provides utilities to orginize readers-writer access to individual cells of your collection.
//!
//! # [`LockerRoom`]
//! The central feature of the crate is implemented by this structure. More specifically, it provides such functionality:
//! 1. Shared readers access to single cell of collection with [`read_cell`](LockerRoom::read_cell);
//! 2. Exclusive writer access to single cell of collection with [`write_cell`](LockerRoom::write_cell);
//! 3. Exclusive writer access to whole collection with [`lock_room`](LockerRoom::lock_room).
//! 
//! ## Example
//! ```
//! # use std::{thread, sync::Arc};
//! # use lockerroom::LockerRoom;
//! let v = vec![0, 1, 2, 3, 4, 5];
//! let locker_room: LockerRoom<_> = v.into();
//! let locker_room = Arc::new(locker_room);
//! thread::scope(|scope| {
//!     scope.spawn(|| *locker_room.write_cell(0).unwrap() += 1);
//!     scope.spawn(|| *locker_room.write_cell(0).unwrap() += 2);
//! });
//! assert_eq!(3, *locker_room.read_cell(0).unwrap());
//! ```
//!
//! ## Deadlock example
//! Carefully block multiple cells in one scope. Otherwise, situation like this may occur:
//! ```text
//! // Thread 1                            |  // Thread 2
//! let _w1 = locker_room.write_cell(0);   |
//!                                        |  let _w1 = locker_room.write_cell(1);
//! // will block
//! let _w2 = locker_room.write_cell(1);   |
//!                                        |  // will deadlock
//!                                        |  let _w2 = locker_room.write_cell(0);
//! ```
//!
//! ## Collections?
//! By default you can create `LockerRoom` from [`array`], [`Vec`], [`VecDeque`](std::collections::VecDeque), 
//! [`HashMap`](std::collections::HashMap) and [`BTreeMap`](std::collections::BTreeMap).
//!
//! But the crate provides trait, by which implementing to your collection, you can make it compatible with `LockerRoom`.
//!
//! # [`Collection`]
//! Crucial part of the crate that helps your collection to be compatible with `LockerRoom`. 
//! 
//! Just implement it into your collection and everything will work!
//!
//! # Example
//! Let's implement the trait for the struct from [`Index`](std::ops::Index)'s [example](https://doc.rust-lang.org/std/ops/trait.Index.html#examples):
//! ```
//! # use std::{sync::RwLock, borrow::Borrow};
//! # use lockerroom::{Collection, ShadowLocksCollection};
//! enum Nucleotide {
//!     C,
//!     A,
//!     G,
//!     T,
//! }
//! 
//! # #[derive(Default)]
//! struct NucleotideCount {
//!     pub a: usize,
//!     pub c: usize,
//!     pub g: usize,
//!     pub t: usize,
//! }
//! 
//! impl Collection for NucleotideCount {
//!     type Output = usize;
//!     type Idx = Nucleotide;
//!     type ShadowLocks = NucleotideShadowLocks;
//! 
//!     fn index(&self, index: impl Borrow<Self::Idx>) -> Option<&Self::Output> {
//!         Some(match index.borrow() {
//!             Nucleotide::A => &self.a,
//!             Nucleotide::C => &self.c,
//!             Nucleotide::G => &self.g,
//!             Nucleotide::T => &self.t,
//!         })
//!     }
//! 
//!     fn index_mut(&mut self, index: impl Borrow<Self::Idx>) -> Option<&mut Self::Output> {
//!         Some(match index.borrow() {
//!             Nucleotide::A => &mut self.a,
//!             Nucleotide::C => &mut self.c,
//!             Nucleotide::G => &mut self.g,
//!             Nucleotide::T => &mut self.t,
//!         })
//!     }
//! 
//!     fn indices(&self) -> impl Iterator<Item = Self::Idx> {
//!         [Nucleotide::A, Nucleotide::C, Nucleotide::G, Nucleotide::T].into_iter()
//!     }
//! 
//!     fn shadow_locks(&self) -> Self::ShadowLocks {
//!         Default::default()
//!     }
//! }
//!
//! # #[derive(Default)]
//! struct NucleotideShadowLocks {
//!     a: RwLock<()>,
//!     c: RwLock<()>,
//!     g: RwLock<()>,
//!     t: RwLock<()>,
//! }
//! 
//! impl ShadowLocksCollection for NucleotideShadowLocks {
//!     type Idx = Nucleotide;
//! 
//!     fn index(&self, index: impl Borrow<Self::Idx>) -> Option<&RwLock<()>> {
//!         Some(match index.borrow() {
//!             Nucleotide::A => &self.a,
//!             Nucleotide::C => &self.c,
//!             Nucleotide::G => &self.g,
//!             Nucleotide::T => &self.t,
//!         })
//!     }
//! 
//!     fn update_indices(&mut self, _indices: impl Iterator<Item = Self::Idx>) {
//!         // No need to reindex because NucleotideShadowLocks has static structure.
//!     }
//! }
//! ```

mod collection;
mod guard;

use std::{borrow::Borrow, cell::UnsafeCell, marker::PhantomData, sync::RwLock};

pub use collection::{Collection, ShadowLocksCollection};
pub use guard::{ReadCellGuard, RoomGuard, WriteCellGuard};

/// Provides readers-writer lock for each indexed cell or exclusive write access to whole collection.
///
/// Works with any collection that implements [`Collection`].
/// ```
/// # use std::{thread, sync::Arc};
/// # use lockerroom::LockerRoom;
/// let v = vec![0, 1, 2, 3, 4, 5];
/// let locker_room: LockerRoom<_> = v.into();
/// let locker_room = Arc::new(locker_room);
/// thread::scope(|scope| {
///     scope.spawn(|| *locker_room.write_cell(0).unwrap() += 1);
///     scope.spawn(|| *locker_room.write_cell(0).unwrap() += 2);
/// });
/// assert_eq!(3, *locker_room.read_cell(0).unwrap());
/// ```
pub struct LockerRoom<T>
where
    T: Collection,
{
    collection: UnsafeCell<T>,
    global_lock: RwLock<()>,
    index_locks: UnsafeCell<T::ShadowLocks>,
    phantom: PhantomData<T::Idx>,
}

unsafe impl<T: Collection> Sync for LockerRoom<T> {}

impl<'a, T> LockerRoom<T>
where
    T: Collection,
{
    /// Locks cell at the index with shared read access, blocking the current thread until it can be acquired.
    ///
    /// This function will return `None` if there is no cell with such index.
    ///
    /// Returns an RAII guard which will release this thread's shared access once it is dropped.
    pub fn read_cell(&'a self, index: impl Borrow<T::Idx>) -> Option<ReadCellGuard<'a, T>> {
        let global_lock_guard = self
            .global_lock
            .read()
            .unwrap_or_else(|err| err.into_inner());
        let index_locks = unsafe { &*self.index_locks.get() };
        let index_lock_guard = index_locks
            .index(index.borrow())?
            .read()
            .unwrap_or_else(|err| err.into_inner());
        let collection = unsafe { &*self.collection.get() };
        collection
            .index(index)
            .map(|v| ReadCellGuard::new(v, global_lock_guard, index_lock_guard))
    }

    /// Locks cell at the index with exclusive write access, blocking the current thread until it can be acquired.
    ///
    /// This function will return `None` if there is no cell with such index.
    ///
    /// Returns an RAII guard which will release this thread's exclusive write access once it is dropped.
    pub fn write_cell(&'a self, index: impl Borrow<T::Idx>) -> Option<WriteCellGuard<'a, T>> {
        let global_lock_guard = self
            .global_lock
            .read()
            .unwrap_or_else(|err| err.into_inner());
        let index_locks = unsafe { &*self.index_locks.get() };
        let index_lock_guard = index_locks
            .index(index.borrow())?
            .write()
            .unwrap_or_else(|err| err.into_inner());
        let collection = unsafe { &mut *self.collection.get() };
        collection
            .index_mut(index)
            .map(|v| WriteCellGuard::new(v, global_lock_guard, index_lock_guard))
    }

    /// Exclusively locks whole collection with right access.
    ///
    /// No cell locks can be acquired by other threads when locked whole collection.
    ///
    /// Returns an RAII guard which will release this thread's exclusive write access once it is dropped.
    pub fn lock_room(&'a self) -> RoomGuard<'a, T> {
        let global_lock_guard = self
            .global_lock
            .write()
            .unwrap_or_else(|err| err.into_inner());
        let index_locks = unsafe { &mut *self.index_locks.get() };
        let collection = unsafe { &mut *self.collection.get() };
        RoomGuard::new(collection, index_locks, global_lock_guard)
    }

    /// Consumes this `LockerRoom`, returning the underlying data.
    pub fn into_inner(self) -> T {
        self.collection.into_inner()
    }
}

impl<T> From<T> for LockerRoom<T>
where
    T: Collection,
{
    fn from(value: T) -> Self {
        let index_locks = value.shadow_locks();
        Self {
            collection: UnsafeCell::new(value),
            global_lock: Default::default(),
            index_locks: UnsafeCell::new(index_locks),
            phantom: Default::default(),
        }
    }
}

#[cfg(test)]
mod test {
    use std::{ops::DerefMut, sync::Arc, thread};

    use crate::LockerRoom;

    #[test]
    fn t() {
        let len = 9999;
        let v: Vec<_> = (0..len).collect();
        let locker_room: Arc<LockerRoom<_>> = Arc::new(v.into());
        thread::scope(|scope| {
            for _ in 0..len {
                scope.spawn(|| {
                    for i in 0..len {
                        *locker_room.write_cell(i).unwrap() += i;
                    }
                });
            }
            scope.spawn(|| {
                for i in 0..len {
                    let mut guard = locker_room.lock_room();
                    let v = guard.deref_mut();
                    v.resize(i + len + 1, i);
                }
            });
        });
        let v = Arc::into_inner(locker_room).unwrap().into_inner();
        for i in 0..len {
            assert_eq!(i * (len + 1), v[i]);
        }
        for i in 0..len {
            assert_eq!(i, v[i + len]);
        }
    }
}
