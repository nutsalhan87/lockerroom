//! Guards for different locking types.

use std::ops::{Deref, DerefMut};

use tokio::sync::{RwLockReadGuard, RwLockWriteGuard};

use crate::{Collection, ShadowLocksCollectionAsync};

/// RAII structure used to release the shared read access of a cell lock when dropped.
///
/// This structure is created by the [`read_cell`](crate::LockerRoomAsync::read_cell) methods on [`LockerRoomAsync`](crate::LockerRoomAsync).
pub struct ReadCellGuard<'a, T>
where
    T: Collection,
{
    value: &'a T::Output,
    // For dropping and, after that, unlocking.
    #[allow(dead_code)]
    cell_rwlock_read_guard: RwLockReadGuard<'a, ()>,
    // For dropping and, after that, unlocking. But it stands after cell guard because of order of dropping.
    #[allow(dead_code)]
    global_rwlock_read_guard: RwLockReadGuard<'a, ()>,
}

impl<'a, T> ReadCellGuard<'a, T>
where
    T: Collection,
{
    pub(crate) fn new(
        value: &'a T::Output,
        global_rwlock_read_guard: RwLockReadGuard<'a, ()>,
        cell_rwlock_read_guard: RwLockReadGuard<'a, ()>,
    ) -> Self {
        Self {
            value,
            global_rwlock_read_guard,
            cell_rwlock_read_guard,
        }
    }
}

impl<'a, T> Deref for ReadCellGuard<'a, T>
where
    T: Collection,
{
    type Target = T::Output;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

/// RAII structure used to release the exclusive write access of a cell lock when dropped.
///
/// This structure is created by the [`write_cell`](crate::LockerRoomAsync::write_cell) methods on [`LockerRoomAsync`](crate::LockerRoomAsync).
pub struct WriteCellGuard<'a, T>
where
    T: Collection,
{
    value: &'a mut T::Output,
    // For dropping and, after that, unlocking.
    #[allow(dead_code)]
    cell_rwlock_write_guard: RwLockWriteGuard<'a, ()>,
    // For dropping and, after that, unlocking. But it stands after cell guard because of order of dropping.
    #[allow(dead_code)]
    global_rwlock_read_guard: RwLockReadGuard<'a, ()>,
}

impl<'a, T> WriteCellGuard<'a, T>
where
    T: Collection,
{
    pub(crate) fn new(
        value: &'a mut T::Output,
        global_rwlock_read_guard: RwLockReadGuard<'a, ()>,
        cell_rwlock_write_guard: RwLockWriteGuard<'a, ()>,
    ) -> Self {
        Self {
            value,
            global_rwlock_read_guard,
            cell_rwlock_write_guard,
        }
    }
}

impl<'a, T> Deref for WriteCellGuard<'a, T>
where
    T: Collection,
{
    type Target = T::Output;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a, T> DerefMut for WriteCellGuard<'a, T>
where
    T: Collection,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

/// RAII structure used to release the exclusive write access of a whole collection lock when dropped.
///
/// This structure is created by the [`lock_room`](crate::LockerRoomAsync::lock_room) methods on [`LockerRoomAsync`](crate::LockerRoomAsync).
pub struct RoomGuard<'a, T>
where
    T: Collection,
{
    collection: &'a mut T,
    index_locks: &'a mut T::ShadowLocksAsync,
    #[allow(dead_code)]
    global_rwlock_write_guard: RwLockWriteGuard<'a, ()>,
}

impl<'a, T> RoomGuard<'a, T>
where
    T: Collection,
{
    pub(crate) fn new(
        collection: &'a mut T,
        index_locks: &'a mut T::ShadowLocksAsync,
        global_rwlock_write_guard: RwLockWriteGuard<'a, ()>,
    ) -> Self {
        Self {
            collection,
            index_locks,
            global_rwlock_write_guard,
        }
    }
}

impl<'a, T> Deref for RoomGuard<'a, T>
where
    T: Collection,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.collection
    }
}

impl<'a, T> DerefMut for RoomGuard<'a, T>
where
    T: Collection,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.collection
    }
}

impl<'a, T> Drop for RoomGuard<'a, T>
where
    T: Collection,
{
    fn drop(&mut self) {
        self.index_locks.update_indices(self.collection.indices());
    }
}
