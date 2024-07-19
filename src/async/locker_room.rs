use std::{borrow::Borrow, cell::UnsafeCell, marker::PhantomData};

use tokio::sync::RwLock;

use super::{Collection, ReadCellGuard, RoomGuard, ShadowLocksCollection, WriteCellGuard};

/// Provides readers-writer lock for each indexed cell or exclusive write access to whole collection.
/// Same as [`LockerRoom`](crate::LockerRoom) but async.
///
/// Works with any collection that implements [`Collection`].
/// ```
/// # use std::sync::Arc;
/// # use lockerroom::LockerRoomAsync;
/// # use tokio::{task::spawn, join};
/// # tokio_test::block_on(async  {
/// let v = vec![0, 1, 2, 3, 4, 5];
/// let locker_room: LockerRoomAsync<_> = v.into();
/// let locker_room = Arc::new(locker_room);
///
/// let locker_room_cloned = Arc::clone(&locker_room);
/// let join1 = spawn(async move { *locker_room_cloned.write_cell(0).await.unwrap() += 1 });
///
/// let locker_room_cloned = Arc::clone(&locker_room);
/// let join2 = spawn(async move { *locker_room_cloned.write_cell(0).await.unwrap() += 2 });
///
/// join!(join1, join2);
/// assert_eq!(3, *locker_room.read_cell(0).await.unwrap());
/// # });
/// ```
pub struct LockerRoomAsync<T>
where
    T: Collection,
{
    collection: UnsafeCell<T>,
    global_lock: RwLock<()>,
    index_locks: UnsafeCell<T::ShadowLocks>,
    phantom: PhantomData<T::Idx>,
}

unsafe impl<T: Collection> Sync for LockerRoomAsync<T> {}

impl<'a, T> LockerRoomAsync<T>
where
    T: Collection,
{
    /// Locks cell at the index with shared read access, causing the current task to yield until the lock has been acquired.
    ///
    /// This function will return `None` if there is no cell with such index.
    ///
    /// Returns an RAII guard which will release this thread's shared access once it is dropped.
    pub async fn read_cell(
        &'a self,
        index: impl Borrow<T::Idx> + Send,
    ) -> Option<ReadCellGuard<'a, T>> {
        let global_lock_guard = self.global_lock.read().await;
        let index_locks = unsafe { &*self.index_locks.get() };
        let index_lock_guard = index_locks.index(index.borrow())?.read().await;
        let collection = unsafe { &*self.collection.get() };
        collection
            .index(index)
            .map(|v| ReadCellGuard::new(v, global_lock_guard, index_lock_guard))
    }

    /// Locks cell at the index with exclusive write access, causing the current task to yield until the lock has been acquired.
    ///
    /// This function will return `None` if there is no cell with such index.
    ///
    /// Returns an RAII guard which will release this thread's exclusive write access once it is dropped.
    pub async fn write_cell(
        &'a self,
        index: impl Borrow<T::Idx> + Send,
    ) -> Option<WriteCellGuard<'a, T>> {
        let global_lock_guard = self.global_lock.read().await;
        let index_locks = unsafe { &*self.index_locks.get() };
        let index_lock_guard = index_locks.index(index.borrow())?.write().await;
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
    pub async fn lock_room(&'a self) -> RoomGuard<'a, T> {
        let global_lock_guard = self.global_lock.write().await;
        let index_locks = unsafe { &mut *self.index_locks.get() };
        let collection = unsafe { &mut *self.collection.get() };
        RoomGuard::new(collection, index_locks, global_lock_guard)
    }

    /// Consumes this `LockerRoomAsync`, returning the underlying data.
    pub fn into_inner(self) -> T {
        self.collection.into_inner()
    }
}

impl<T> From<T> for LockerRoomAsync<T>
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
    use std::{ops::DerefMut, sync::Arc};

    use tokio::task::JoinSet;

    use super::LockerRoomAsync;

    #[test]
    fn t() {
        const LEN: usize = 999;
        let v: Vec<_> = (0..LEN).collect();
        let locker_room: Arc<LockerRoomAsync<_>> = Arc::new(v.into());

        tokio_test::block_on(async {
            let mut join_set = JoinSet::new();

            for _ in 0..LEN {
                let locker_room_cloned = Arc::clone(&locker_room);
                join_set.spawn(async move {
                    for i in 0..LEN {
                        *locker_room_cloned.write_cell(i).await.unwrap() += i;
                    }
                    drop(locker_room_cloned);
                });
            }

            let locker_room_cloned = Arc::clone(&locker_room);
            join_set.spawn(async move {
                for i in 0..LEN {
                    let mut guard = locker_room_cloned.lock_room().await;
                    let v = guard.deref_mut();
                    v.resize(i + LEN + 1, i);
                }
                drop(locker_room_cloned);
            });

            while let Some(_) = join_set.join_next().await {}
        });

        let v = Arc::into_inner(locker_room).unwrap().into_inner();
        for i in 0..LEN {
            assert_eq!(i * (LEN + 1), v[i]);
        }
        for i in 0..LEN {
            assert_eq!(i, v[i + LEN]);
        }
    }
}
