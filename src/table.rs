use std::hash::BuildHasher;

// pub enum Entry {
//     Vacant(VacantEntry),
//     Occupied,
// }

// pub struct VacantEntry {
//     hash: u64,
//     key: K,
//     // table: &'a mut Table,
// }

// pub struct OccupiedEntry {
//     hash: u64,
//     elem: (K, V),
//     // table: &'a mut Table,
// }

#[derive(Debug)]
pub struct Table<K, V, H, const S: usize>
where
    H: BuildHasher,
{
    hash_builder: H,
    storage: [Option<(K, V)>; S],
}

impl<K, V, H, const S: usize> Table<K, V, H, S>
where
    H: BuildHasher,
{
    pub fn with_hasher(hasher: H) -> Self {
        Self {
            hash_builder: hasher,
            storage: [const { None }; S],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::hash::RandomState;

    #[test]
    fn my_very_specific_test() {
        let table = Table::<String, String, RandomState, 10>::with_hasher(RandomState::default());

        dbg!(&table);
    }
}
