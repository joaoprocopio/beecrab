use std::hash::{BuildHasher, Hash};
use std::ptr::NonNull;

/// Cache-friendly, fixed-capacity, open-addressing hash table with linear probing.
/// TODO: SIMD.

pub type Index = usize;

pub enum Entry<'a, K, V, H, const S: usize> {
    Vacant(VacantEntry<'a, K, V, H, S>),
    Occupied(OccupiedEntry<'a, K, V, H, S>),
}

pub struct VacantEntry<'a, K, V, H, const S: usize> {
    hash: u64,
    key: K,
    table: &'a mut Table<'a, K, V, H, S>,
}

pub struct OccupiedEntry<'a, K, V, H, const S: usize> {
    elem: Bucket<(K, V)>,
    table: &'a mut Table<'a, K, V, H, S>,
}

pub struct Bucket<T> {
    ptr: NonNull<T>,
}

pub struct Table<'a, K, V, H, const S: usize> {
    hasher: H,
    mask: usize,
    table: Box<[Option<Entry<'a, K, V, H, S>>; S]>,
}

impl<'a, K: Hash + PartialEq, V, H: BuildHasher, const S: usize> Table<'a, K, V, H, S> {
    pub fn with_hasher(hasher: H) -> Self {
        Self {
            hasher,
            table: Box::new([const { None }; S]),
            mask: S - 1,
        }
    }
}

mod tests {

    use crate::{fnv::FnvBuildHasher, metrics::Aggregate};

    use super::*;
    use std::collections::hash_map;

    fn v() {
        let mut hm = hash_map::HashMap::<&'_ [u8], u64>::new();
        match hm.entry(&[1]) {
            hash_map::Entry::Occupied(mut some) => {
                some.get_mut();
            }
            hash_map::Entry::Vacant(none) => {
                none.insert(1);
            }
        };
    }

    fn t() {
        const SIZE: usize = 1 << 14;
        let mut t = Table::<'_, &'_ [u8], Aggregate, FnvBuildHasher, SIZE>::with_hasher(
            FnvBuildHasher::new(),
        );
    }
}
