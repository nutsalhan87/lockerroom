#![feature(doc_cfg)]

//! Crate provides utilities to orginize readers-writer access to individual cells of your collection.
//!
//! # [`LockerRoom`] and [`LockerRoomAsync`]
//! The central features of the crate is implemented by these structures. More specifically, they provide such functionality:
//! 1. Shared readers access to single cell of collection with [`LockerRoom::read_cell`] and [`LockerRoomAsync::read_cell`];
//! 2. Exclusive writer access to single cell of collection with [`LockerRoom::write_cell`] and [`LockerRoomAsync::write_cell`];
//! 3. Exclusive writer access to whole collection with [`LockerRoom::lock_room`] and [`LockerRoomAsync::lock_room`].
//!
//! But `LockerRoomAsync` is optional -- you need to enable feature `async` to use it. It depends on
//! [`tokio`](https://docs.rs/tokio/latest/tokio/index.html)'s [`RwLock`](https://docs.rs/tokio/latest/tokio/sync/struct.RwLock.html).
//!
//! ## `LockerRoom` example
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
//! ## `LockerRoomAsync` example
//! ```
//! # use std::sync::Arc;
//! # use lockerroom::LockerRoomAsync;
//! # use tokio::{task::spawn, join};
//! # tokio_test::block_on(async  {
//! let v = vec![0, 1, 2, 3, 4, 5];
//! let locker_room: LockerRoomAsync<_> = v.into();
//! let locker_room = Arc::new(locker_room);
//!
//! let locker_room_cloned = Arc::clone(&locker_room);
//! let join1 = spawn(async move { *locker_room_cloned.write_cell(0).await.unwrap() += 1 });
//!
//! let locker_room_cloned = Arc::clone(&locker_room);
//! let join2 = spawn(async move { *locker_room_cloned.write_cell(0).await.unwrap() += 2 });
//!
//! join!(join1, join2);
//! assert_eq!(3, *locker_room.read_cell(0).await.unwrap());
//! # });
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
//! By default you can create `LockerRoom` and `LockerRoomAsync` from [`array`], [`Vec`], [`VecDeque`](std::collections::VecDeque),
//! [`HashMap`](std::collections::HashMap) and [`BTreeMap`](std::collections::BTreeMap).
//!
//! But the crate provides traits, by which implementing to your collection, you can make it compatible with `LockerRoom` and `LockerRoomAsync`.
//!
//! # `Collection`
//! Crucial part of the crate that helps your collection to be compatible with `LockerRoom`.
//!
//! Just implement it into your collection and everything will work!
//!
//! In fact, there is two different `Collection`s: [`sync::Collection`] and [`async::Collection`].
//! First one is for the `LockerRoom` and the second one is for the `LockerRoomAsync`. So you need to implement both of them for your collection to use it with
//! both LockerRooms.\
//! That's bad design. But I'll fix it in next versions of the crate.
//!
//! ## Example
//! Let's implement the trait for the struct from [`Index`](std::ops::Index)'s [example](https://doc.rust-lang.org/std/ops/trait.Index.html#examples):
//! ```
//! # use std::{sync::RwLock, borrow::Borrow};
//! # use lockerroom::sync::{Collection, ShadowLocksCollection};
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

#[cfg(any(feature = "async", doc))]
#[doc(cfg(feature = "async"))]
pub mod r#async;
pub mod sync;

#[cfg(any(feature = "async", doc))]
#[doc(cfg(feature = "async"))]
pub use r#async::LockerRoomAsync;
pub use sync::LockerRoom;
