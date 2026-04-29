use std::hash::{BuildHasher, Hash};

const S: usize = 2 << 14;
const M: usize = S - 1;

#[inline]
fn make_index(hash: usize) -> usize {
    hash & M
}

#[inline]
fn make_hash<Q, H>(hash_builder: &H, val: &Q) -> u64
where
    Q: Hash,
    H: BuildHasher,
{
    hash_builder.hash_one(val)
}

#[inline]
fn make_next_probe(index: usize) -> usize {
    index + 1
}

#[derive(Debug)]
pub struct Table<K, V, H> {
    hash_builder: H,
    table: Vec<Option<(K, V)>>,
}

impl<K, V, H> Table<K, V, H>
where
    K: Eq + Hash,
    H: BuildHasher,
{
    pub fn with_hasher(hasher: H) -> Self {
        let mut table = Vec::with_capacity(S);
        table.resize_with(S, || None);

        Self {
            hash_builder: hasher,
            table: table,
        }
    }

    pub fn get(&self, key: K) -> Option<&V> {
        let hash = make_hash(&self.hash_builder, &key);
        let index = make_index(hash as usize);

        loop {
            match self.table[index].as_ref() {
                Some(inner_elem) if inner_elem.0 == key => return Some(&inner_elem.1),
                _ => return None,
            }
        }
    }

    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        let hash = make_hash(&self.hash_builder, &key);
        let mut index = make_index(hash as usize);

        loop {
            match self.table[index].as_ref() {
                Some(inner_elem) if inner_elem.0 == key => {
                    return self.table[index].as_mut().map(|kv| &mut kv.1);
                }
                Some(_) => index = make_next_probe(index),
                None => return None,
            }
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        let hash = make_hash(&self.hash_builder, &key);
        let mut index = make_index(hash as usize);

        loop {
            match self.table[index].as_ref() {
                Some(elem_inner) if elem_inner.0 != key => {
                    index = make_next_probe(index);
                }
                _ => {
                    // In this branch `elem_inner.0 == key` or `None`.
                    self.table[index] = Some((key, value));
                    return;
                }
            }
        }
    }
}

impl<K, V, H> IntoIterator for Table<K, V, H> {
    type Item = Option<(K, V)>;
    type IntoIter = std::vec::IntoIter<Option<(K, V)>>;

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

    #[test]
    fn insert_with_collisions() {
        let mut table = Table::<&[u8], Aggregate, RandomState>::with_hasher(RandomState::default());

        table.insert(b"jac", Aggregate::new(1));
        table.insert(b"pedro", Aggregate::new(2));

        assert_eq!(table.get(b"jac"), Some(&Aggregate::new(1)));
        assert_eq!(table.get(b"pedro"), Some(&Aggregate::new(2)));

        table.insert(b"jac", Aggregate::new(16));

        assert_eq!(table.get(b"jac"), Some(&Aggregate::new(16)));
    }
}
