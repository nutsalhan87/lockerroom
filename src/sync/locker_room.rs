use std::{borrow::Borrow, cell::UnsafeCell, marker::PhantomData, sync::RwLock};

use super::{Collection, ReadCellGuard, RoomGuard, ShadowLocksCollection, WriteCellGuard};

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

    use super::LockerRoom;

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
