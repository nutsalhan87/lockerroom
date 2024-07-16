# Locker Room
### Readers-writer access to individual cells of your collection!

## LockerRoom
The central feature of the crate is implemented by this structure. More specifically, it provides such functionality:
1. Shared readers access to single cell of collection with `read_cell` method;
2. Exclusive writer access to single cell of collection with `write_cell` method;
3. Exclusive writer access to whole collection with `lock_room` method.

### Example
```rust
let v = vec![0, 1, 2, 3, 4, 5];
let locker_room: LockerRoom<_> = v.into();
let locker_room = Arc::new(locker_room);
thread::scope(|scope| {
    scope.spawn(|| *locker_room.write_cell(0).unwrap() += 1);
    scope.spawn(|| *locker_room.write_cell(0).unwrap() += 2);
});
assert_eq!(3, *locker_room.read_cell(0).unwrap());
```
### Deadlock example
Carefully block multiple cells in one scope. Otherwise, situation like this may occur:
```text
// Thread 1                            |  // Thread 2
let _w1 = locker_room.write_cell(0);   |
                                       |  let _w1 = locker_room.write_cell(1);
// will block
let _w2 = locker_room.write_cell(1);   |
                                       |  // will deadlock
                                       |  let _w2 = locker_room.write_cell(0);
```
### Collections?
By default you can create `LockerRoom` from `array`, `Vec`, `VecDeque`, `HashMap` and `BTreeMap`.
But the crate provides trait, by which implementing to your collection, you can make it compatible with `LockerRoom`.
## Collection
Crucial part of the crate that helps your collection to be compatible with `LockerRoom`. 

Just implement it into your collection and everything will work!
## Example
Let's implement the trait for the struct from `Index`'s [example](https://doc.rust-lang.org/std/ops/trait.Index.html#examples):
```rust
enum Nucleotide {
    C,
    A,
    G,
    T,
}

struct NucleotideCount {
    pub a: usize,
    pub c: usize,
    pub g: usize,
    pub t: usize,
}

impl Collection for NucleotideCount {
    type Output = usize;
    type Idx = Nucleotide;
    type ShadowLocks = NucleotideShadowLocks;

    fn index(&self, index: impl Borrow<Self::Idx>) -> Option<&Self::Output> {
        Some(match index.borrow() {
            Nucleotide::A => &self.a,
            Nucleotide::C => &self.c,
            Nucleotide::G => &self.g,
            Nucleotide::T => &self.t,
        })
    }

    fn index_mut(&mut self, index: impl Borrow<Self::Idx>) -> Option<&mut Self::Output> {
        Some(match index.borrow() {
            Nucleotide::A => &mut self.a,
            Nucleotide::C => &mut self.c,
            Nucleotide::G => &mut self.g,
            Nucleotide::T => &mut self.t,
        })
    }

    fn indices(&self) -> impl Iterator<Item = Self::Idx> {
        [Nucleotide::A, Nucleotide::C, Nucleotide::G, Nucleotide::T].into_iter()
    }

    fn shadow_locks(&self) -> Self::ShadowLocks {
        Default::default()
    }
}
struct NucleotideShadowLocks {
    a: RwLock<()>,
    c: RwLock<()>,
    g: RwLock<()>,
    t: RwLock<()>,
}

impl ShadowLocksCollection for NucleotideShadowLocks {
    type Idx = Nucleotide;

    fn index(&self, index: impl Borrow<Self::Idx>) -> Option<&RwLock<()>> {
        Some(match index.borrow() {
            Nucleotide::A => &self.a,
            Nucleotide::C => &self.c,
            Nucleotide::G => &self.g,
            Nucleotide::T => &self.t,
        })
    }

    fn update_indices(&mut self, _indices: impl Iterator<Item = Self::Idx>) {
        // No need to reindex because NucleotideShadowLocks has static structure.
    }
}
```