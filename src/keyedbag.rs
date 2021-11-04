use std::cmp::Eq;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// A HashMap where each key is associated with a set of values.
/// Inserting different values for the same keys associates each
/// unique value with the given key.
pub struct KeyedBag<K: Eq + Hash, V: Eq + Hash> {
    /// The underlying container that matches key -> { value0, value1, ... }.
    container: HashMap<K, HashSet<V>>,

    /// Empty set for returns from method get. It is pretty silly that we have
    /// to create a new empty set for each keyed bag, but from what I can tell
    /// Rust does not let me create static variables that take generic
    /// parameters (i.e. HashSet<V> *inside* method get). So this is the best I
    /// can come up with right now.
    empty_set: HashSet<V>,
}

impl<K: Eq + Hash, V: Eq + Hash> KeyedBag<K, V> {
    /// Create a new, empty KeyedBag.
    pub fn new() -> KeyedBag<K, V> {
        KeyedBag {
            container: HashMap::new(),
            empty_set: HashSet::new(),
        }
    }

    /// Insert value associated with key.
    pub fn insert(&mut self, key: K, value: V) {
        self.container.entry(key).or_default().insert(value);
    }

    /// Return the set of values associated with key. If no value was associated
    /// with the given key, this method returns an empty hash set.
    pub fn get(&self, key: &K) -> &HashSet<V> {
        match self.container.get(key) {
            None => &self.empty_set,
            Some(value) => value,
        }
    }

    /// Return a copy of all keys in this KeyedBag.
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.container.keys()
    }
}

#[cfg(test)]
mod tests {
    use crate::keyedbag::KeyedBag;
    use std::collections::HashSet;

    fn count<T>(it: &mut dyn Iterator<Item = T>) -> usize {
        it.map(|_| 1).sum()
    }

    #[test]
    fn constructing_empty_bag() {
        let bag: KeyedBag<i32, i32> = KeyedBag::new();
        assert!(bag.keys().next().is_none());
        assert_eq!(count(&mut bag.keys()), 0);
    }

    #[test]
    fn insert_single() {
        let mut bag: KeyedBag<usize, usize> = KeyedBag::new();

        let start: usize = 0;
        let end: usize = 100;

        for i in start..end {
            bag.insert(i, i);
            assert_eq!(count(&mut bag.keys()), i + 1);
        }

        for key in bag.keys() {
            let values = bag.get(key);
            assert_eq!(values.len(), 1);

            let first = values.iter().next().unwrap();
            assert_eq!(*first, *key);

        }
    }

    #[test]
    fn insert_multiple() {
        let mut bag: KeyedBag<i32, i32> = KeyedBag::new();

        bag.insert(1, 1);
        bag.insert(1, 10);
        bag.insert(1, 100);

        bag.insert(2, 200);
        bag.insert(2, 20);
        bag.insert(2, 2);

        bag.insert(3, 1);
        bag.insert(3, 10);
        bag.insert(3, 100);

        bag.insert(4, 1);
        bag.insert(4, 10);
        bag.insert(4, 100);

        assert_eq!(count(&mut bag.keys()), 4);

        {
            let mut expected_for_key_1: HashSet<i32> = HashSet::new();

            expected_for_key_1.insert(1);
            expected_for_key_1.insert(10);
            expected_for_key_1.insert(100);

            assert_eq!(&expected_for_key_1, bag.get(&1));
        }

        {
            let mut expected_for_key_2: HashSet<i32> = HashSet::new();

            expected_for_key_2.insert(200);
            expected_for_key_2.insert(20);
            expected_for_key_2.insert(2);

            assert_eq!(&expected_for_key_2, bag.get(&2));
        }

        {
            let mut expected_for_key_3: HashSet<i32> = HashSet::new();

            expected_for_key_3.insert(100);
            expected_for_key_3.insert(10);
            expected_for_key_3.insert(1);

            assert_eq!(&expected_for_key_3, bag.get(&3));
        }

        {
            let mut expected_for_key_4: HashSet<i32> = HashSet::new();

            expected_for_key_4.insert(100);
            expected_for_key_4.insert(10);
            expected_for_key_4.insert(1);

            assert_eq!(&expected_for_key_4, bag.get(&4));
        }
    }
}
