use std::array;
use std::hash::{BuildHasher, Hash};

const S: usize = 2 << 12;
const M: usize = S - 1;

#[inline]
fn make_index(hash: u64) -> usize {
    hash as usize & M
}

fn make_hash<Q, H>(hash_builder: &H, val: &Q) -> u64
where
    Q: Hash,
    H: BuildHasher,
{
    hash_builder.hash_one(val)
}

#[derive(Debug)]
pub struct Table<K, V, H> {
    hash_builder: H,
    table: Box<[Option<(K, V)>; S]>,
}

impl<K, V, H> Table<K, V, H>
where
    K: Hash,
    H: BuildHasher,
{
    pub fn with_hasher(hasher: H) -> Self {
        Self {
            hash_builder: hasher,
            table: Box::new([const { None }; S]),
        }
    }

    pub fn get(&self, key: K) -> Option<&V> {
        let hash = make_hash(&self.hash_builder, &key);
        let index = make_index(hash);

        let elem = self.table[index].as_ref();

        elem.and_then(|el| Some(&el.1))
    }

    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        let hash = make_hash(&self.hash_builder, &key);
        let index = make_index(hash);

        let elem = self.table[index].as_mut();

        elem.and_then(|el| Some(&mut el.1))
    }

    pub fn insert(&mut self, key: K, value: V) {
        let hash = make_hash(&self.hash_builder, &key);
        let index = make_index(hash);

        self.table[index] = Some((key, value));
    }
}

impl<K, V, H> IntoIterator for Table<K, V, H> {
    type Item = Option<(K, V)>;
    type IntoIter = array::IntoIter<Option<(K, V)>, S>;

    fn into_iter(self) -> Self::IntoIter {
        self.table.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::Aggregate;
    use std::hash::RandomState;

    #[test]
    fn insert() {
        let mut table = Table::<&[u8], Aggregate, RandomState>::with_hasher(RandomState::default());

        table.insert(b"jac", Aggregate::new(1));

        assert_eq!(table.get(b"jac"), Some(&Aggregate::new(1)));
    }
}
